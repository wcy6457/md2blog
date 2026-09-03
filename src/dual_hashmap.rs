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

impl Default for DualHashmap {
    fn default() -> Self {
        Self::new()
    }
}

impl DualHashmap {
    pub fn new() -> DualHashmap {
        DualHashmap {
            file_path_to_page_list: HashMap::new(),
            uri_path_to_page_list: HashMap::new(),
        }
    }

    /**
    同时按实例内容对两个hashmap同时插入的比较底层的方法

    目前来看一般只应该在服务器第一次启动的时候、重新遍历加载markdown的时候调用或被其他封装好的类调用

    Date：2026.9.3
    */
    pub fn insert_by_page(&mut self, page: Arc<Page>) {
        self.file_path_to_page_list
            .insert(page.file_path.clone(), Arc::clone(&page));
        self.uri_path_to_page_list
            .insert(page.uri_path.clone(), Arc::clone(&page));
    }
}

pub trait DualHashmapArcSwapExt {
    fn insert_by_page(&self, page: Page);
    fn get_page_by_uri_path(&self, uri_path: &str) -> Option<Arc<Page>>;
    fn get_page_by_file_path(&self, file_path: &str) -> Option<Arc<Page>>;
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

    fn get_page_by_file_path(&self, file_path: &str) -> Option<Arc<Page>> {
        match self.load().file_path_to_page_list.get(file_path) {
            Some(s) => Some(Arc::clone(s)),
            None => None,
        }
    }

    /**
    内部做大量“合规”验证的更新方法
    */
    fn update_page_by_file_path(&self, file_path: &str) -> Result<(), String> {
        match self.get_page_by_file_path(file_path) {
            //1先查双重hashmap里有无关于这个文件路径的记录
            Some(page) => {
                //1查到了
                let uri_path = &page.uri_path; //1从查到的实例里取出uri_path

                match self.load().uri_path_to_page_list.get(uri_path) {
                    //2再从前面的uri_path查另外一个hashmap有无记录
                    Some(_) => {
                        //2查到了
                        self.rcu(|dual_hashmap| {
                            //两个hashmap都查到了开始对咱们这个ArcSwap做rcu
                            let mut dual_hashmap = DualHashmap::clone(dual_hashmap); //复制现在的样子
                            dual_hashmap.file_path_to_page_list.remove(file_path);
                            dual_hashmap.uri_path_to_page_list.remove(uri_path); //更新复制的里面的两个hashmap，移除相关记录
                            dual_hashmap.insert_by_page(Arc::new(Page::new(file_path))); //对复制的加入新的实例
                            dual_hashmap //返回这个复制，完成rcu
                        });
                        Ok(())
                    }
                    None => Err(
                        //2没查到          目前来看恐怕真的是预料不到的错误了吧。。。Date：2026.9.3
                        "发生了未知的意外，这一定是因为异步操作的bug导致的。请反馈开发者。"
                            .to_string(),
                    ),
                }
            }
            None => {
                //1没查到
                let page = Page::new(file_path);
                match self.load().uri_path_to_page_list.get(&page.uri_path) {
                    //2直接建一个新的实例
                    Some(_) => Err(
                        //但是查到了
                        "正在载入的文件所配置的uri_path已经存在页面，服务器拒绝加载！".to_string(),
                    ),
                    None => {
                        //没查到才能插入
                        self.insert_by_page(page);
                        Ok(())
                    }
                }
            }
        }
    }
}
