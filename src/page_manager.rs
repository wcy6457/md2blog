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
    pub dual_hashmap: Arc<ArcSwap<DualHashmap>>,
    test_style: TestStyle,
}

impl PageManager {
    pub fn init() -> PageManager {
        let dual_hashmap = DualHashmap::new();

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

                    dual_hashmap.insert_by_page(Page::new(file_path));
                }
                Err(e) => {
                    eprintln!("加载文件时出错：{:?}", e);
                    exit(1);
                }
            }
        }

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

        PageManager {
            dual_hashmap,
            test_style,
        }
    }

    pub fn get_test_style_clone(&self) -> TestStyle {
        self.test_style.clone()
    }
}
