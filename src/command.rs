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
                Ok(_) => self.handle(input.trim_end()).await,
                Err(e) => eprintln!("读取命令时发生了错误：{e}"),
            }
        }
    }

    async fn handle(&self, command: &str) {
        if let Some(file_path) = command.strip_prefix("reload ") {
            let page_manager = self.page_manager_store.load_full();
            match page_manager.update_page_by_file_path(file_path.trim()).await {
                true => println!("reload success"),
                false => eprintln!("In reload: file not found or file not md: {} , or maybe you need \"refresh\"", file_path)
            }
        } else if command == "refresh" {
            let file_manager = self.page_manager_store.load();
            let file_manager = Arc::new(file_manager.refreshed());
            self.page_manager_store.store(file_manager);
            println!("refresh finished~");
        } else if command == "exit" {
            println!("stop~");
            exit(0);
        } else {
            println!("杂鱼，这点指令都输不对~");
        }
    }
}
