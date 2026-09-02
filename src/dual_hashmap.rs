use crate::page::Page;
use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::sync::Arc;

type FilePathToPageList = HashMap<String, Arc<Page>>;
type UriPathToPageList = HashMap<String, Arc<Page>>;

pub struct DualHashmap {
    file_path_to_page_list: FilePathToPageList,
    uri_path_to_page_list: UriPathToPageList,
}

impl Clone for DualHashmap {
    fn clone(&self) -> Self {
        DualHashmap {
            file_path_to_page_list: self.file_path_to_page_list.clone(),
            uri_path_to_page_list: self.uri_path_to_page_list.clone(),
        }
    }
}

impl DualHashmap {
    pub fn new() -> Arc<ArcSwap<DualHashmap>> {
        Arc::new(ArcSwap::from_pointee(DualHashmap {
            file_path_to_page_list: HashMap::new(),
            uri_path_to_page_list: HashMap::new(),
        }))
    }

    fn insert_by_page(&mut self, page: Arc<Page>) {
        self.file_path_to_page_list
            .insert(Arc::clone(&page).file_path.clone(), Arc::clone(&page));
        self.uri_path_to_page_list
            .insert(Arc::clone(&page).uri_path.clone(), Arc::clone(&page));
    }
}

pub trait DualHashmapArcSwapExt {
    fn insert_by_page(&self, page: Page);
    fn get_page_by_uri_path(&self, uri_path: &str) -> Option<Arc<Page>>;
    fn get_page_by_path_path(&self, file_path: &str) -> Option<Arc<Page>>;
    fn update_page_by_file_path(&self, file_path: &str) -> Result<(), String>;
}
impl DualHashmapArcSwapExt for ArcSwap<DualHashmap> {
    fn insert_by_page(&self, page: Page) {
        let page = Arc::new(page);
        self.rcu(|dual_hashmap| {
            let mut dual_hashmap = DualHashmap::clone(dual_hashmap);
            dual_hashmap.insert_by_page(Arc::clone(&page));
            dual_hashmap
        });
    }

    fn get_page_by_uri_path(&self, uri_path: &str) -> Option<Arc<Page>> {
        match self.load().uri_path_to_page_list.get(uri_path) {
            Some(s) => Some(Arc::clone(s)),
            None => None,
        }
    }

    fn get_page_by_path_path(&self, file_path: &str) -> Option<Arc<Page>> {
        match self.load().file_path_to_page_list.get(file_path) {
            Some(s) => Some(Arc::clone(s)),
            None => None,
        }
    }

    fn update_page_by_file_path(&self, file_path: &str) -> Result<(), String> {
        match self.get_page_by_path_path(file_path) {
            Some(page) => {
                let uri_path = page.uri_path.clone();

                match self.load().uri_path_to_page_list.get(uri_path.as_str()) {
                    Some(_) => {
                        self.rcu(|dual_hashmap| {
                            let mut dual_hashmap = DualHashmap::clone(dual_hashmap);
                            dual_hashmap.file_path_to_page_list.remove(file_path);
                            dual_hashmap.uri_path_to_page_list.remove(uri_path.as_str());
                            dual_hashmap.insert_by_page(Arc::new(Page::new(file_path.to_string())));
                            dual_hashmap
                        });
                        Ok(())
                    }
                    None => Err(
                        "发生了未知的意外，这一定是因为异步操作的bug导致的。请反馈开发者。"
                            .to_string(),
                    ),
                }
            }
            None => {
                self.insert_by_page(Page::new(file_path.to_string()));
                Ok(())
            }
        }
    }
}
