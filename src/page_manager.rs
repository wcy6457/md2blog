use crate::page::Page;
use axum::body::Bytes;
use axum::http::StatusCode;
use glob::glob;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::Mutex;

type FilePathToPageList = HashMap<String, Arc<Mutex<Page>>>;
type UriPathToPageList = HashMap<String, Arc<Mutex<Page>>>;
type TestStyle = Result<Bytes, (StatusCode, Bytes)>;


//好丑陋todo
pub struct PageManager {
    file_path_to_page_list: FilePathToPageList,
    uri_path_to_page_list: UriPathToPageList,
    test_style: TestStyle,
}

impl PageManager {
    pub fn init() -> PageManager {
        //read css from hard_disk
        let test_style = Self::build_test_style();

        Self::load_pages(test_style)
    }

    pub fn refreshed(&self) -> PageManager {
        Self::load_pages(self.test_style.clone())
    }

    fn build_test_style() -> Result<Bytes, (StatusCode, Bytes)> {
        match read_to_string("test/style.css") {
            Ok(css) => Ok(Bytes::from(css)),
            Err(e) => {
                eprintln!("在读取CSS文件的时候发生了错误：{}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Bytes::from(e.to_string()),
                ))
            }
        }
    }

    fn load_pages(test_style: TestStyle) -> PageManager {
        let mut file_path_to_page_list = HashMap::new();
        let mut uri_path_to_page_list = HashMap::new();

        for entry in glob("test/*.md").expect("Failed to read glob pattern") {
            match entry {
                Ok(file_path) => {
                    let file_path = match file_path.into_os_string().into_string() {
                        Ok(file_path) => file_path.replace('\\', "/"),
                        Err(err) => {
                            eprintln!("搜寻文件时无法把路径转换成 String：{:?}", err);
                            continue;
                        }
                    };
                    let uri_path = match Self::read_uri_path_from_file_path(&file_path) {        // format!("/{}", file_path.trim_end_matches(".md"));
                        Ok(s) => {
                            s
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            file_path.clone()
                        }
                    };
                    let page = Arc::new(Mutex::new(Page::new(file_path.clone())));

                    file_path_to_page_list.insert(file_path, Arc::clone(&page));
                    uri_path_to_page_list.insert(uri_path, page);
                }
                Err(e) => {
                    eprintln!("加载文件时出错：{:?}", e);
                    exit(1);
                }
            }
        }

        PageManager {
            file_path_to_page_list,
            uri_path_to_page_list,
            test_style,
        }
    }

    pub async fn update_page_by_file_path(&self, file_path: &str) -> bool {
        let file_path = file_path.replace('\\', "/");
        let page = self.file_path_to_page_list.get(&file_path);
        match page {
            Some(page) => {
                page.lock().await.update_html();
                true
            }
            None => {
                false
            }
        }
    }

    pub fn get_page_by_uri_path(&self, uri_path: &str) -> Option<Arc<Mutex<Page>>> {
        self.uri_path_to_page_list.get(uri_path).cloned()
    }

    pub fn get_test_style_clone(&self) -> TestStyle {
        self.test_style.clone()
    }

    fn read_uri_path_from_file_path(file_path: &String) -> Result<String, String> {
        match read_to_string(Path::new(file_path)) {
            Ok(s) => {
                match s.lines().filter(|line| { line.contains("uri_path:") }).collect::<Vec<&str>>().first() {
                    Some(s) => {
                        Ok(s.trim().trim_start_matches("uri_path:").to_string())
                    }
                    None => {
                        Err(format!("在{file_path}中找不到关于uri_path的设置。"))
                    }
                }
            }
            Err(e) => Err(e.to_string())
        }
    }
}
