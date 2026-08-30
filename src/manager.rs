use crate::test::Test;
use axum::body::Bytes;
use axum::http::StatusCode;
use glob::glob;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::Mutex;

type Page = Arc<Mutex<Test>>;
type FileList = HashMap<String, Page>;
type RouteList = HashMap<String, Page>;
type TestStyle = Result<Bytes, (StatusCode, Bytes)>;

pub struct FileManager {
    file_list: FileList,
    route_list: RouteList,
    test_style: TestStyle,
}

impl FileManager {
    pub fn init() -> FileManager {
        //read css from hard_disk
        let test_style = match read_to_string("test/style.css") {
            Ok(css) => Ok(Bytes::from(css)),
            Err(e) => {
                eprintln!("在读取CSS文件的时候发生了错误：{}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Bytes::from(e.to_string()),
                ))
            }
        };

        Self::load_test_files(test_style)
    }

    pub fn refreshed(&self) -> FileManager {
        Self::load_test_files(self.test_style.clone())
    }

    fn load_test_files(test_style: TestStyle) -> FileManager {
        let mut file_list = HashMap::new();
        let mut route_list = HashMap::new();

        for entry in glob("test/*.md").expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let path = match path.into_os_string().into_string() {
                        Ok(path) => path.replace('\\', "/"),
                        Err(path) => {
                            println!("搜寻文件时无法把路径转换成 String：{:?}", path);
                            continue;
                        }
                    };
                    let route_path = format!("/{}", path.trim_end_matches(".md"));
                    let page = Arc::new(Mutex::new(Test::new(path.clone())));

                    file_list.insert(path, Arc::clone(&page));
                    route_list.insert(route_path, page);
                }
                Err(e) => {
                    println!("加载文件时出错：{:?}", e);
                    exit(1);
                }
            }
        }

        FileManager {
            file_list,
            route_list,
            test_style,
        }
    }

    pub async fn update_html(&self, path: &str) {
        let path = path.replace('\\', "/");
        let file = self.file_list.get(&path).cloned();
        match file {
            Some(file) => file.lock().await.update_html(),
            None => eprintln!(
                "In reload: file not found or file not md: {} , or maybe you need \"refresh\"",
                path
            ),
        }
    }

    pub fn file_by_route(&self, route: &str) -> Option<Page> {
        self.route_list.get(route).cloned()
    }

    pub fn test_style(&self) -> TestStyle {
        self.test_style.clone()
    }
}
