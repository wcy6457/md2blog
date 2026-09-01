use crate::page_manager::PageManager;
use arc_swap::ArcSwap;
use std::process::exit;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};

pub struct CommandHandler {
    page_manager_store: Arc<ArcSwap<PageManager>>,
}

impl CommandHandler {
    pub fn new(page_manager_store: Arc<ArcSwap<PageManager>>) -> Self {
        Self {
            page_manager_store
        }
    }

    pub async fn run(self) {
        let mut input = String::new();
        let mut stdin = BufReader::new(io::stdin());

        loop {
            input.clear();
            match stdin.read_line(&mut input).await {
                Ok(0) => return,
                Ok(_) => {
                    println!("-----");
                    self.handle(input.trim_end()).await;
                    println!("-----");
                }
                Err(e) => eprintln!("读取命令时发生了错误：{e}"),
            }
        }
    }

    async fn handle(&self, command: &str) {
        if let Some(file_path) = command.strip_prefix("reload ") {
            println!("正在重新加载文件{file_path}......");
            println!("todo~~~");
        } else if command == "refresh" {
            println!("正在重新加载所有文件......");
            let page_manager = Arc::new(PageManager::init());
            self.page_manager_store.store(page_manager);
        } else if command == "exit" {
            println!("服务器关闭中~");
            exit(0);
        } else {
            println!("杂鱼，这点指令都输不对~");
        }
    }
}
