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
}


pub trait DualHashmapArcSwapExt {
    fn insert_by_page(&self, page: Page);
    fn get_page_by_uri_path(&self, uri_path: &str) -> Option<Arc<Page>>;
}
impl DualHashmapArcSwapExt for ArcSwap<DualHashmap> {
    fn insert_by_page(&self, page: Page) {
        let page = Arc::new(page);
        self.rcu(|dual_hashmap| {
            let mut dual_hashmap = DualHashmap::clone(dual_hashmap);
            dual_hashmap.file_path_to_page_list.insert(Arc::clone(&page).file_path.clone(), Arc::clone(&page));
            dual_hashmap.uri_path_to_page_list.insert(Arc::clone(&page).uri_path.clone(), Arc::clone(&page));
            dual_hashmap
        });
    }

    fn get_page_by_uri_path(&self, uri_path: &str) -> Option<Arc<Page>> {
        match self.load().uri_path_to_page_list.get(uri_path) {
            Some(s) => Some(Arc::clone(s)),
            None => None
        }
    }
}