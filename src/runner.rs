use crate::command::CommandHandler;
use crate::manager::FileManager;
use crate::test::Test;
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
    file_manager_store: Arc<ArcSwap<FileManager>>,
}

impl Runner {
    pub fn init(file_manager: FileManager) -> Self {
        Runner {
            file_manager_store: Arc::new(ArcSwap::from_pointee(file_manager)),
        }
    }

    pub async fn run_server(runner: Runner) {
        tokio::spawn(CommandHandler::new(Arc::clone(&runner.file_manager_store)).run());

        axum::serve(
            match TcpListener::bind("0.0.0.0:2233").await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("发生了错误：{}", e.kind());
                    exit(1);
                }
            },
            Self::get_handler(runner.file_manager_store),
        )
        .await
        .unwrap();
    }

    fn get_handler(file_manager_store: Arc<ArcSwap<FileManager>>) -> Router {
        let test_style = {
            let file_manager = file_manager_store.load();
            file_manager.test_style()
        };

        return Router::new()
            .route(
                "/test/style.css",
                get(move || {
                    let test_style = test_style.clone();
                    async move {
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
                    }
                }),
            )
            .fallback(fallback)
            .with_state(file_manager_store);

        async fn fallback(
            uri: Uri,
            State(file_manager_store): State<Arc<ArcSwap<FileManager>>>,
        ) -> Response<Body> {
            let file = {
                let file_manager = file_manager_store.load();
                file_manager.file_by_route(uri.path())
            };

            if let Some(file) = file {
                return file.lock().await.build_response();
            }

            Test::build_404_response()
        }
    }
}
