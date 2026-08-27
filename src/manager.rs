use crate::md_file_path_to_html;
use crate::test::Test;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode, Uri};
use axum::Router;
use glob::glob;
use std::collections::HashMap;
use std::fmt::format;
use std::process::exit;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub struct FileManager {
    file_list: Arc<Mutex<HashMap<Arc<String>, Arc<Mutex<Test>>>>>,
}

impl FileManager {
    pub async fn init() -> Arc<Mutex<FileManager>> {
        let map = HashMap::new();
        let map = Arc::new(Mutex::new(map));
        let temp = Arc::new(Mutex::new(FileManager { file_list: map }));
        let temp = Arc::clone(&temp);
        temp.lock().await.load_test_file().await;
        temp
    }

    pub async fn refresh(&self) {
        self.file_list.lock().await.clear();
        self.load_test_file().await;
        println!("refresh finished~")
    }

    async fn load_test_file(&self) {
        for entry in glob("test/*.md").expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let path = match path.into_string() {
                        Ok(s) => {
                            s
                        }
                        Err(e) => {
                            println!("搜寻文件将路径转换为String时出错：{:?}", e);
                            String::new()
                        }
                    };
                    let path = Arc::new(path.replace('\\', "/"));
                    let path1 = Arc::clone(&path);
                    let path2 = Arc::clone(&path);
                    let path3 = Arc::clone(&path);
                    self.file_list.lock().await.insert(path1, Arc::new(Mutex::new(Test::new(path2, md_file_path_to_html(path3)))));
                }
                Err(e) => {
                    println!("{:?}", e);
                    exit(1);
                }
            }
        }
    }

    pub async fn add(&self, path: Arc<String>, point: Arc<Mutex<Test>>) {
        self.file_list.lock().await.insert(path, point);
    }

    pub async fn update_html(&self, path: &str) {
        let path = Arc::new(path.replace('\\', "/"));
        let file = self.file_list.lock().await.get(&path).cloned();
        match file {
            Some(file) => file.lock().await.update_html(),
            None => eprintln!("In reload: file not found or file not md: {} , or maybe you need \"refresh\"", path),
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
