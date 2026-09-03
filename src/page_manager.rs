use crate::dual_hashmap::{DualHashmap, DualHashmapArcSwapExt};
use crate::page::Page;
use arc_swap::ArcSwap;
use axum::body::Bytes;
use axum::http::StatusCode;
use glob::glob;
use std::fs::read_to_string;
use std::process::exit;
use std::sync::Arc;

type TestStyle = Result<Bytes, (StatusCode, Bytes)>;

pub struct PageManager {
    dual_hashmap: Arc<ArcSwap<DualHashmap>>,
    test_style: TestStyle,
}

impl PageManager {
    pub fn init() -> PageManager {
        //read Markdown files from disk
        let dual_hashmap = DualHashmap::new();
        let dual_hashmap = load_pages_from_disk_into_map(dual_hashmap);

        //read css from disk
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

        PageManager {
            dual_hashmap: Arc::new(ArcSwap::from_pointee(dual_hashmap)),
            test_style,
        }
    }
}

/**
目前是遍历预设好的相对路径：test/\* 下的markdown文件。其他功能todo

Date：2026.9.3
*/
fn load_pages_from_disk_into_map(mut dual_hashmap: DualHashmap) -> DualHashmap {
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

                dual_hashmap.insert_by_page(Arc::new(Page::new(&file_path)));
            }
            Err(e) => {
                eprintln!("加载文件时出错：{:?}", e);
                exit(1);
            }
        }
    }
    dual_hashmap
}

pub trait PageManagerStoreExt {
    fn get_test_style_clone(&self) -> TestStyle;
    fn add_page(&self, page: Page);
    fn get_page_by_uri_path(&self, uri_path: &str) -> Option<Arc<Page>>;
    fn get_page_by_file_path(&self, file_path: &str) -> Option<Arc<Page>>;
    fn update_page_by_file_path(&self, file_path: &str) -> Result<(), String>;
    fn refresh(&self);
}

impl PageManagerStoreExt for ArcSwap<PageManager> {
    fn get_test_style_clone(&self) -> TestStyle {
        self.load().test_style.clone()
    }

    fn add_page(&self, page: Page) {
        self.load().dual_hashmap.insert_by_page(page);
    }

    fn get_page_by_uri_path(&self, uri_path: &str) -> Option<Arc<Page>> {
        self.load().dual_hashmap.get_page_by_uri_path(uri_path)
    }

    fn get_page_by_file_path(&self, file_path: &str) -> Option<Arc<Page>> {
        self.load().dual_hashmap.get_page_by_file_path(file_path)
    }

    fn update_page_by_file_path(&self, file_path: &str) -> Result<(), String> {
        self.load().dual_hashmap.update_page_by_file_path(file_path)
    }

    fn refresh(&self) {
        let dual_hashmap = DualHashmap::new();
        let dual_hashmap = load_pages_from_disk_into_map(dual_hashmap);
        self.load().dual_hashmap.store(Arc::new(dual_hashmap));
    }
}
