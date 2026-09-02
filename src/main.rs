pub mod command;
pub mod page_manager;
pub mod runner;
pub mod page;
pub mod dual_hashmap;

use crate::page_manager::PageManager;
use crate::runner::Runner;
#[tokio::main]
async fn main() {
    println!("服务器正在启动......");

    let page_manager = PageManager::init();

    let runner = Runner::init(page_manager);

    Runner::run_server(runner).await;

    println!("bye~");
}
