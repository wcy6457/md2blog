use crate::test::Test;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode, Uri};
use axum::Router;
use std::collections::HashMap;
use std::fmt::format;
use std::process::exit;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub struct FileManager {
    file_list: Arc<Mutex<HashMap<Arc<String>, Arc<Mutex<Test>>>>>,
}

impl Default for FileManager {
    fn default() -> Self {
        let map = HashMap::new();
        let map = Arc::new(Mutex::new(map));
        FileManager {
            file_list: map
        }
    }
}

impl FileManager {
    pub async fn add(&mut self, path: Arc<String>, point: Arc<Mutex<Test>>) {
        self.file_list.lock().await.insert(path, point);
    }

    pub async fn update_html(&self, path: &str) {
        let path = Arc::new(path.replace('\\', "/"));
        let file = self.file_list.lock().await.get(&path).cloned();
        match file {
            Some(file) => file.lock().await.update_html(),
            None => eprintln!("In reload: file not found: {}", path),
        }
    }

    pub async fn run_server(file_manager: Arc<Mutex<Self>>) {
        let handler = file_manager.lock().await.get_handler();
        axum::serve(match TcpListener::bind("0.0.0.0:2233").await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("发生了错误：{}", e.kind());
                exit(1);
            }
        }, handler).await.unwrap();
    }

    fn get_handler(&self) -> Router {
        async fn fallback(uri: Uri, file_list: State<Arc<Mutex<HashMap<Arc<String>, Arc<Mutex<Test>>>>>>) -> Response<Body> {
            for file in file_list.lock().await.iter() {
                let (path, file) = file;
                if uri.eq(format(format_args!("/{}", path.trim_end_matches(".md"))).as_str()) {
                    return file.lock().await.build_response(StatusCode::OK);
                }
            }
            Test::build_404_response()
        }
        Router::new().fallback(fallback).with_state(Arc::clone(&self.file_list))
    }
}


// impl<'a> Iterator for FileManager<'a> {
//     type Item = ();?
//
//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
