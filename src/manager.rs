use crate::test::Test;
use axum::body::Bytes;
use axum::http::StatusCode;
use glob::glob;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) type UriPath = Arc<String>;
type FileList = Mutex<HashMap<UriPath, Arc<Mutex<Test>>>>;
type TestStyle = Arc<Result<Bytes, (StatusCode, Bytes)>>;
type FilePath = Arc<String>;

pub struct FileManager {
    pub file_list: FileList,
    pub test_style: TestStyle,
}

impl FileManager {
    pub async fn init() -> Arc<FileManager> {
        let map = Mutex::new(HashMap::new());

        //read css from hard_disk
        let test_style = Arc::new(match read_to_string("test/style.css") {
            Ok(css) => Ok(Bytes::from(css)),
            Err(e) => {
                eprintln!("在读取CSS文件的时候发生了错误：{}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Bytes::from(e.to_string())))
            }
        });

        let file_manager = FileManager {
            file_list: map,
            test_style,
        };

        let temp = Arc::clone(&Arc::new(file_manager));
        let temp2 = Arc::clone(&temp);
        Self::load_test_file(temp).await;

        temp2
    }

    pub async fn refresh(file_manager: Arc<FileManager>) {
        file_manager.file_list.lock().await.clear();
        Self::load_test_file(file_manager).await;
        println!("refresh finished~");
    }

    async fn load_test_file(file_manager: Arc<FileManager>) {
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
                    let uri_path: FilePath = Arc::clone(&path);
                    let path3 = Arc::clone(&path);
                    file_manager.file_list.lock().await.insert(uri_path, Arc::new(Mutex::new(Test::new(path3))));
                }
                Err(e) => {
                    println!("加载文件时出错：{:?}", e);
                    exit(1);
                }
            }
        }
    }

    pub async fn update_html(&self, path: &str) {
        let path = Arc::new(path.replace('\\', "/"));
        let file = self.file_list.lock().await.get(&path).cloned();
        match file {
            Some(file) => file.lock().await.update_html(),
            None => eprintln!("In reload: file not found or file not md: {} , or maybe you need \"refresh\"", path),
        }
    }
}