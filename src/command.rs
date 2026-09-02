use crate::dual_hashmap::DualHashmapArcSwapExt;
use crate::page_manager::PageManager;
use arc_swap::ArcSwap;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};

pub struct CommandHandler {
    page_manager_store: Arc<ArcSwap<PageManager>>,
}

impl CommandHandler {
    pub fn new(page_manager_store: Arc<ArcSwap<PageManager>>) -> Self {
        Self { page_manager_store }
    }

    pub async fn run(self) {
        let mut input = String::new();
        let mut stdin = BufReader::new(io::stdin());

        println!("服务器已经上线！");

        loop {
            input.clear();
            match stdin.read_line(&mut input).await {
                Ok(0) => return,
                Ok(_) => {
                    println!("-----");
                    if self.handle(input.trim_end()).await {
                        return;
                    };
                    println!("-----");
                }
                Err(e) => eprintln!("读取命令时发生了错误：{e}"),
            }
        }
    }

    async fn handle(&self, command: &str) -> bool {
        if let Some(file_path) = command.strip_prefix("reload ") {
            println!("正在重新加载文件{file_path}......");
            match self
                .page_manager_store
                .load_full()
                .dual_hashmap
                .update_page_by_file_path(file_path)
            {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e),
            };
            false
        } else if command == "refresh" {
            println!("正在重新加载所有文件......");
            let page_manager = Arc::new(PageManager::init());
            self.page_manager_store.store(page_manager);
            false
        } else if command == "exit" {
            println!("服务器关闭中~");
            true
        } else {
            println!("杂鱼，这点指令都输不对~");
            false
        }
    }
}
