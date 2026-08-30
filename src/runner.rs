use crate::manager::FileManager;
use crate::test::Test;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode, Uri};
use axum::routing::get;
use axum::Router;
use std::fmt::format;
use std::process::exit;
use std::sync::Arc;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;

pub struct Runner {
    arc_file_manager: Arc<FileManager>,
}

impl Runner {
    pub fn init(arc_file_manager: Arc<FileManager>) -> Self {
        Runner {
            arc_file_manager,
        }
    }

    pub async fn run_server(runner: Arc<Runner>) {
        tokio::spawn(Self::commandline_handler(Arc::clone(&runner.arc_file_manager)));

        axum::serve(match TcpListener::bind("0.0.0.0:2233").await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("发生了错误：{}", e.kind());
                exit(1);
            }
        }, Self::get_handler(runner)).await.unwrap();
    }

    async fn commandline_handler(file_manager: Arc<FileManager>) {
        let mut input = String::new();
        loop {
            let mut stdin = BufReader::new(io::stdin());
            stdin.read_line(&mut input).await.unwrap_or_else(|e| {
                println!("{}", e);
                0
            });
            let temp = input.trim_end();

            // println!("输入了{}", input);
            if temp.starts_with("reload ") {
                let temp = match temp.strip_prefix("reload ") {
                    Some(temp) => {
                        temp
                    }
                    None => {
                        temp
                    }
                };
                let temp1 = Arc::clone(&file_manager);
                temp1.update_html(temp.trim()).await;
            } else if temp == "refresh" {
                let temp1 = Arc::clone(&file_manager);
                FileManager::refresh(temp1).await;
            } else if temp == "exit" {
                println!("stop~");
                exit(0);
            } else {
                println!("杂鱼，这点指令都输不对~");
            }
            input.clear();
        }
    }

    fn get_handler(runner: Arc<Runner>) -> Router {
        let test_style = Arc::clone(&runner.arc_file_manager.test_style);

        return Router::new()
            .route("/test/style.css", get(move || {
                let test_style = Arc::clone(&test_style);
                async move {
                    match &*test_style {
                        Ok(css) => Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", "text/css; charset=utf-8")
                            .header("cache-control", "no-store")
                            .body(Body::from(css.clone()))
                            .unwrap(),
                        Err((code, reason)) => Response::builder()
                            .status(*code)
                            .header("content-type", "text/plain; charset=utf-8")
                            .body(Body::from(reason.clone()))
                            .unwrap(),
                    }
                }
            }))
            .fallback(fallback)
            .with_state(Arc::clone(&runner));

        async fn fallback(uri: Uri, runner: State<Arc<Runner>>) -> Response<Body> {
            for file in runner.arc_file_manager.file_list.lock().await.iter() {
                let (path, file) = file;
                if uri.eq(format(format_args!("/{}", path.trim_end_matches(".md"))).as_str()) {
                    return file.lock().await.build_response();
                }
            }
            Test::build_404_response()
        }
    }
}