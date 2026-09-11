use crate::config::Config;
use crate::dual_hashmap::{DualHashmap, DualHashmapError};
use crate::page::Page;
use arc_swap::ArcSwap;
use axum::body::Bytes;
use axum::http::StatusCode;
use glob::glob;
use std::fs::read_to_string;
use std::process::exit;
use std::sync::Arc;

type TestStyle = Result<Bytes, (StatusCode, Bytes)>;

#[derive(Debug)]
pub enum PageManagerError {
    Message(String),
    UriPath(String),
}

impl From<DualHashmapError> for PageManagerError {
    fn from(error: DualHashmapError) -> Self {
        Self::Message(match error {
            DualHashmapError::DuplicateFile(file_path) => {
                format!("文件 {file_path} 已加载，已跳过重复加载")
            }
            DualHashmapError::DuplicateUri {
                uri_path,
                file_path,
            } => {
                format!("路由 {uri_path} 已被文件 {file_path} 占用，已拒绝加载")
            }
        })
    }
}

pub struct PageManager {
    config: Config,
    dual_hashmap: Arc<ArcSwap<DualHashmap>>,
    test_style: TestStyle,
}

impl PageManager {
    pub fn init(config: Config) -> PageManager {
        //read Markdown files from disk
        let dual_hashmap = load_pages_from_disk(config.get_root());

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
fn load_pages_from_disk(root_path: &str) -> DualHashmap {
    let mut dual_hashmap = DualHashmap::new();
    for entry in glob(&format!("{root_path}/**/*.md")).expect("Failed to read glob pattern") {
        match entry {
            Ok(file_path) => {
                let file_path = match file_path.into_os_string().into_string() {
                    Ok(file_path) => file_path.replace('\\', "/"),
                    Err(err) => {
                        eprintln!("搜寻文件时无法把路径转换成 String：{:?}", err);
                        continue;
                    }
                };

                match Page::build(&file_path, root_path) {
                    Ok(page) => {
                        let page = Arc::new(page);
                        if let Err(error) = dual_hashmap.insert_by_page(page) {
                            if let PageManagerError::Message(message) = error.into() {
                                eprintln!("加载文件 {file_path} 时，{message}");
                            }
                        }
                    }
                    Err(uri_path) => {
                        eprintln!(
                            "加载文件 {} 时，路由 {} 属于保留路由，已跳过加载",
                            file_path, uri_path
                        );
                    }
                }
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
    fn get_config(&self) -> Config;
    fn get_test_style_clone(&self) -> TestStyle;
    fn add_page(&self, page: Page) -> Result<(), PageManagerError>;
    fn get_page_by_uri_path(&self, uri_path: &str) -> Option<Arc<Page>>;
    fn get_page_by_file_path(&self, file_path: &str) -> Option<Arc<Page>>;
    fn update_page_by_file_path(&self, file_path: &str) -> Result<(), PageManagerError>;
    fn refresh(&self);
}

impl PageManagerStoreExt for ArcSwap<PageManager> {
    fn get_config(&self) -> Config {
        self.load().config.clone()
    }

    fn get_test_style_clone(&self) -> TestStyle {
        self.load().test_style.clone()
    }

    fn add_page(&self, page: Page) -> Result<(), PageManagerError> {
        let manager = self.load();
        let page = Arc::new(page);
        let mut result = Ok(());
        manager.dual_hashmap.rcu(|current| {
            let mut next = DualHashmap::clone(current);
            result = next.insert_by_page(Arc::clone(&page)).map_err(Into::into);
            next
        });
        result
    }

    fn get_page_by_uri_path(&self, uri_path: &str) -> Option<Arc<Page>> {
        self.load()
            .dual_hashmap
            .load()
            .get_page_by_uri_path(uri_path)
    }

    fn get_page_by_file_path(&self, file_path: &str) -> Option<Arc<Page>> {
        self.load()
            .dual_hashmap
            .load()
            .get_page_by_file_path(file_path)
    }

    fn update_page_by_file_path(&self, file_path: &str) -> Result<(), PageManagerError> {
        let manager = self.load();
        let file_path = file_path.replace('\\', "/");
        // 磁盘读取只执行一次；RCU 重试时仅修改内存中的双索引。
        let page = Page::build(&file_path, manager.config.get_root()).map(Arc::new);
        let mut result = Ok(());
        manager.dual_hashmap.rcu(|current| {
            let mut next = DualHashmap::clone(current);
            // 从本次快照中移除旧路由，构建失败也必须撤下旧页面。
            next.remove_by_file_path(&file_path);
            result = match &page {
                Ok(page) => next.insert_by_page(Arc::clone(page)).map_err(Into::into),
                Err(uri_path) => Err(PageManagerError::UriPath(uri_path.clone())),
            };
            next
        });
        // 返回最终成功发布的那次修改结果。
        result
    }

    fn refresh(&self) {
        let manager = self.load();
        let dual_hashmap = load_pages_from_disk(manager.config.get_root());
        manager.dual_hashmap.store(Arc::new(dual_hashmap));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
            let root = std::env::temp_dir().join(format!(
                "md2blog-reload-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&root).unwrap();
            std::fs::write(root.join("style.css"), "").unwrap();
            Self(root)
        }

        fn write(&self, name: &str, route: &str) -> String {
            let path = self.0.join(name);
            std::fs::write(&path, format!("uri_path: {route}\n# {name}")).unwrap();
            path.to_str().unwrap().replace('\\', "/")
        }

        fn manager(&self) -> ArcSwap<PageManager> {
            let root = self.0.to_str().unwrap().replace('\\', "/");
            let config = rust_yaml::serde_integration::from_str::<Config>(&format!(
                "content_root: '{root}'\nport: '0'"
            ))
            .unwrap();
            ArcSwap::from_pointee(PageManager::init(config))
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            for name in ["a.md", "b.md", "style.css"] {
                let _ = std::fs::remove_file(self.0.join(name));
            }
            let _ = std::fs::remove_dir(&self.0);
        }
    }

    #[test]
    fn rejected_reload_removes_old_route_and_file_index() {
        let fixture = Fixture::new();
        for route in ["/admin", "/admin/settings", "/api", "/api/posts"] {
            let file = fixture.write("a.md", "/old");
            fixture.write("b.md", "/other");
            let manager = fixture.manager();
            let other = manager.get_page_by_uri_path("/other").unwrap();
            fixture.write("a.md", route);

            assert!(matches!(
                manager.update_page_by_file_path(&file),
                Err(PageManagerError::UriPath(rejected)) if rejected == route
            ));
            assert!(manager.get_page_by_uri_path("/old").is_none());
            assert!(manager.get_page_by_file_path(&file).is_none());
            assert!(manager.get_page_by_uri_path(route).is_none());
            assert!(Arc::ptr_eq(
                &other,
                &manager.get_page_by_uri_path("/other").unwrap()
            ));
            manager.refresh();
            assert!(manager.get_page_by_uri_path("/old").is_none());
            assert!(manager.get_page_by_file_path(&file).is_none());
        }
    }

    #[test]
    fn conflicting_reload_removes_old_page_but_preserves_route_owner() {
        let fixture = Fixture::new();
        let file = fixture.write("a.md", "/old");
        let owner_file = fixture.write("b.md", "/occupied");
        let manager = fixture.manager();
        let owner = manager.get_page_by_uri_path("/occupied").unwrap();
        fixture.write("a.md", "/occupied");

        assert!(matches!(
            manager.update_page_by_file_path(&file),
            Err(PageManagerError::Message(_))
        ));
        assert!(manager.get_page_by_uri_path("/old").is_none());
        assert!(manager.get_page_by_file_path(&file).is_none());
        assert!(Arc::ptr_eq(
            &owner,
            &manager.get_page_by_uri_path("/occupied").unwrap()
        ));
        assert!(Arc::ptr_eq(
            &owner,
            &manager.get_page_by_file_path(&owner_file).unwrap()
        ));
    }

    #[test]
    fn reload_moves_route_updates_content_and_loads_new_file() {
        let fixture = Fixture::new();
        let file = fixture.write("a.md", "/old");
        let manager = fixture.manager();
        fixture.write("a.md", "/new");
        manager.update_page_by_file_path(&file).unwrap();
        assert!(manager.get_page_by_uri_path("/old").is_none());
        let page = manager.get_page_by_uri_path("/new").unwrap();
        assert!(Arc::ptr_eq(
            &page,
            &manager.get_page_by_file_path(&file).unwrap()
        ));

        std::fs::write(&file, "uri_path: /new\n# Updated content").unwrap();
        manager.update_page_by_file_path(&file).unwrap();
        let updated = manager.get_page_by_uri_path("/new").unwrap();
        assert!(!Arc::ptr_eq(&page, &updated));
        assert!(
            String::from_utf8_lossy(&updated.html.as_ref().unwrap().1).contains("Updated content")
        );

        let new_file = fixture.write("b.md", "/added");
        manager.update_page_by_file_path(&new_file).unwrap();
        assert!(manager.get_page_by_uri_path("/added").is_some());
    }
}
