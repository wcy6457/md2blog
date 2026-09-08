use crate::config::Config;
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
    config: Config,
    dual_hashmap: Arc<ArcSwap<DualHashmap>>,
    test_style: TestStyle,
}

impl PageManager {
    pub fn init(config: Config) -> PageManager {
        //read Markdown files from disk
        let dual_hashmap = DualHashmap::new();
        let dual_hashmap =
            load_pages_from_disk_into_map(config.get_root().to_string(), dual_hashmap);

        //read css from disk
        let test_style = match read_to_string(config.get_root().to_string() + "/style.css") {
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
            config,
            dual_hashmap: Arc::new(ArcSwap::from_pointee(dual_hashmap)),
            test_style,
        }
    }
}

/**
配置文件和config实例保存的都是"遍历的起点"

Date：2026.9.5
*/
fn load_pages_from_disk_into_map(root_path: String, mut dual_hashmap: DualHashmap) -> DualHashmap {
    let mut temp = false;
    for entry in glob(&(root_path.clone() + "/**/*.md")).expect("Failed to read glob pattern") {
        match entry {
            Ok(file_path) => {
                let file_path = match file_path.into_os_string().into_string() {
                    Ok(file_path) => file_path.replace('\\', "/"),
                    Err(err) => {
                        eprintln!("搜寻文件时无法把路径转换成 String：{:?}", err);
                        continue;
                    }
                };

                let page = Arc::new(Page::new(&file_path, &root_path));
                if is_reserved_route(&page.uri_path) {
                    eprintln!(
                        "加载文件 {} 时，路由 {} 属于保留路由，已跳过加载",
                        page.file_path, page.uri_path
                    );
                    temp = true;
                } else {
                    dual_hashmap.insert_by_page(page);
                }
            }
            Err(e) => {
                eprintln!("加载文件时出错：{:?}", e);
                exit(1);
            }
        }
    }

    if temp {
        eprintln!("作为保留,已跳过\"/admin\"、\"/api\"的加载");
    }
    dual_hashmap
}

fn is_reserved_route(uri_path: &str) -> bool {
    matches!(uri_path, "/admin" | "/api")
        || uri_path.starts_with("/admin/")
        || uri_path.starts_with("/api/")
}

pub trait PageManagerStoreExt {
    fn get_config(&self) -> Config;
    fn get_test_style_clone(&self) -> TestStyle;
    fn add_page(&self, page: Page);
    fn get_page_by_uri_path(&self, uri_path: &str) -> Option<Arc<Page>>;
    fn get_page_by_file_path(&self, file_path: &str) -> Option<Arc<Page>>;
    fn update_page_by_file_path(&self, file_path: &str) -> Result<(), String>;
    fn refresh(&self);
}

impl PageManagerStoreExt for ArcSwap<PageManager> {
    fn get_config(&self) -> Config {
        self.load().config.clone()
    }

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
        self.load()
            .dual_hashmap
            .update_page_by_file_path(file_path, self.load().config.get_root())
    }

    fn refresh(&self) {
        let dual_hashmap = DualHashmap::new();
        let dual_hashmap =
            load_pages_from_disk_into_map(self.load().config.get_root().to_string(), dual_hashmap);
        self.load().dual_hashmap.store(Arc::new(dual_hashmap));
    }
}
