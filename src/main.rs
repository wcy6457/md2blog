pub mod test;
pub mod manager;
pub mod runner;
pub mod utils;

use crate::manager::FileManager;
use crate::runner::Runner;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let file_manager = FileManager::init().await;

    let runner = Arc::clone(&Arc::new(Runner::init(file_manager)));

    Runner::run_server(runner).await;
}
