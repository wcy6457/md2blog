use crate::command::CommandHandler;
use crate::dual_hashmap::DualHashmapArcSwapExt;
use crate::page::Page;
use crate::page_manager::PageManager;
use arc_swap::ArcSwap;
use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode, Uri};
use axum::routing::get;
use std::process::exit;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct Runner {
    page_manager_store: Arc<ArcSwap<PageManager>>,
}

impl Runner {
    pub fn init(page_manager: PageManager) -> Self {
        Runner {
            page_manager_store: Arc::new(ArcSwap::from_pointee(page_manager)),
        }
    }

    pub async fn run_server(runner: Runner) {
        let server = axum::serve(
            match TcpListener::bind("0.0.0.0:2233").await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("发生了错误：{}", e.kind());
                    exit(1);
                }
            },
            Self::get_handler(Arc::clone(&runner.page_manager_store)),
        );

        server
            .with_graceful_shutdown(async {
                tokio::spawn(CommandHandler::new(runner.page_manager_store).run())
                    .await
                    .unwrap()
            })
            .await
            .unwrap();
    }

    fn get_handler(page_manager_store: Arc<ArcSwap<PageManager>>) -> Router {
        let test_style = {
            let page_manager = page_manager_store.load();
            page_manager.get_test_style_clone()
        };

        return Router::new()
            .route(
                "/test/style.css",
                get(move || async move {
                    match test_style {
                        Ok(css) => Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", "text/css; charset=utf-8")
                            .header("cache-control", "no-store")
                            .body(Body::from(css))
                            .unwrap(),
                        Err((code, reason)) => Response::builder()
                            .status(code)
                            .header("content-type", "text/plain; charset=utf-8")
                            .body(Body::from(reason))
                            .unwrap(),
                    }
                }),
            )
            .fallback(fallback)
            .with_state(page_manager_store);

        async fn fallback(
            uri: Uri,
            State(page_manager_store): State<Arc<ArcSwap<PageManager>>>,
        ) -> Response<Body> {
            match page_manager_store
                .load()
                .dual_hashmap
                .get_page_by_uri_path(uri.path())
            {
                Some(page) => page.build_response().await,
                None => Page::build_404_response(),
            }
        }
    }
}
