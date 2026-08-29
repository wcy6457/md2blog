use crate::manager::FileManager;
use crate::test::Test;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode, Uri};
use axum::routing::get;
use axum::Router;
use std::collections::HashMap;
use std::fmt::format;
use std::process::exit;
use std::sync::Arc;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub struct Runner {
    file_manager: FileManager,
}

impl Runner {
    pub fn init(file_manager: FileManager) -> Self {
        Runner {
            file_manager
        }
    }

    pub async fn run_server(runner: Arc<Self>) {
        tokio::spawn(Self::commandline_handler(Arc::clone(&runner)));

        axum::serve(match TcpListener::bind("0.0.0.0:2233").await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("发生了错误：{}", e.kind());
                exit(1);
            }
        }, runner.get_handler()).await.unwrap();
    }

    async fn commandline_handler(runner: Arc<Self>) {
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
                runner.file_manager.update_html(temp.trim()).await;
            } else if temp == "refresh" {
                runner.file_manager.refresh().await;
            } else if temp == "exit" {
                println!("stop~");
                exit(0);
            } else {
                println!("杂鱼，这点指令都输不对~");
            }
            input.clear();
        }
    }

    fn get_handler(&self) -> Router {
        let test_style = Arc::clone(&self.file_manager.get_test_style());

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
            .with_state(Arc::clone(&self.file_manager.get_file_list()));

        async fn fallback(uri: Uri, file_list: State<Arc<Mutex<HashMap<Arc<String>, Arc<Mutex<Test>>>>>>) -> Response<Body> {
            for file in file_list.lock().await.iter() {
                let (path, file) = file;
                if uri.eq(format(format_args!("/{}", path.trim_end_matches(".md"))).as_str()) {
                    return file.lock().await.build_response(StatusCode::OK);
                }
            }
            Test::build_404_response()
        }
    }
}