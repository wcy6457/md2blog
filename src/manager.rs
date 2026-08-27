use crate::test::Test;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode, Uri};
use axum::Router;
use std::collections::HashMap;
use std::fmt::format;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub struct FileManager {
    file_list: Arc<Mutex<HashMap<Arc<String>, Arc<Mutex<Test>>>>>,
}

impl FileManager {
    pub fn new() -> FileManager {
        let map = HashMap::new();
        let map = Arc::new(Mutex::new(map));
        FileManager {
            file_list: map
        }
    }

    pub async fn add(&mut self, path: Arc<String>, point: Arc<Mutex<Test>>) {
        self.file_list.lock().await.insert(path, point);
    }

    pub async fn run_server(self: Arc<Self>) {
        axum::serve(match TcpListener::bind("0.0.0.0:2233").await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("发生了错误：{}", e.kind());
                exit(1);
            }
        }, Self::get_handler(Arc::clone(&self))).await.unwrap();
    }

    fn get_handler(self: Arc<Self>) -> Router {
        async fn fallback<'a>(uri: Uri, file_list: State<Arc<Mutex<HashMap<&Path, Arc<Mutex<Test>>>>>>) -> Response<Body> {
            for file in file_list.lock().await.iter() {
                match file {
                    (path, file) => {
                        if uri.eq(format(format_args!("/{}", path.as_os_str().to_str().unwrap_or_else(|| { "" }).trim_end_matches(".md"))).as_str()) {
                            return file.lock().await.build_response(StatusCode::OK);
                        }
                    }
                }
            }
            Test::build_404_response()
        }
        Router::new().fallback(fallback).with_state(self.file_list)
    }
}


// impl<'a> Iterator for FileManager<'a> {
//     type Item = ();?
//
//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }