use crate::manager::FileManager;
use arc_swap::ArcSwap;
use std::process::exit;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};

pub struct CommandHandler {
    file_manager_store: Arc<ArcSwap<FileManager>>,
}

impl CommandHandler {
    pub fn new(file_manager_store: Arc<ArcSwap<FileManager>>) -> Self {
        Self { file_manager_store }
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
        if let Some(path) = command.strip_prefix("reload ") {
            let file_manager = self.file_manager_store.load_full();
            file_manager.update_html(path.trim()).await;
        } else if command == "refresh" {
            let file_manager = self.file_manager_store.load();
            let refreshed = Arc::new(file_manager.refreshed());
            self.file_manager_store.store(refreshed);
            println!("refresh finished~");
        } else if command == "exit" {
            println!("stop~");
            exit(0);
        } else {
            println!("杂鱼，这点指令都输不对~");
        }
    }
}
