use crate::page::Page;
use std::collections::HashMap;
use std::sync::Arc;

/// 同一页面的文件路径索引和 URI 索引，所有修改都同时维护两个索引。
#[derive(Clone, Default)]
pub struct DualHashmap {
    file_path_to_page: HashMap<String, Arc<Page>>,
    uri_path_to_page: HashMap<String, Arc<Page>>,
}

#[derive(Debug)]
pub enum DualHashmapError {
    DuplicateFile(String),
    DuplicateUri { uri_path: String, file_path: String },
}

impl DualHashmap {
    pub fn new() -> Self {
        Self::default()
    }

    /// 冲突时不修改任何索引，由调用方决定如何处理错误。
    pub fn insert_by_page(&mut self, page: Arc<Page>) -> Result<(), DualHashmapError> {
        if self.file_path_to_page.contains_key(&page.file_path) {
            return Err(DualHashmapError::DuplicateFile(page.file_path.clone()));
        }
        if let Some(existing) = self.uri_path_to_page.get(&page.uri_path) {
            return Err(DualHashmapError::DuplicateUri {
                uri_path: page.uri_path.clone(),
                file_path: existing.file_path.clone(),
            });
        }
        self.file_path_to_page
            .insert(page.file_path.clone(), Arc::clone(&page));
        self.uri_path_to_page.insert(page.uri_path.clone(), page);
        Ok(())
    }

    pub fn get_page_by_uri_path(&self, uri_path: &str) -> Option<Arc<Page>> {
        self.uri_path_to_page.get(uri_path).cloned()
    }

    pub fn get_page_by_file_path(&self, file_path: &str) -> Option<Arc<Page>> {
        self.file_path_to_page.get(file_path).cloned()
    }

    pub fn remove_by_file_path(&mut self, file_path: &str) -> Option<Arc<Page>> {
        let page = self.file_path_to_page.remove(file_path)?;
        self.uri_path_to_page.remove(&page.uri_path);
        Some(page)
    }
}
