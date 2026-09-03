pub mod command;
pub mod dual_hashmap;
pub mod page;
pub mod page_manager;
pub mod runner;

use crate::page_manager::PageManager;
use crate::runner::Runner;
#[tokio::main]
async fn main() {
    println!("服务器正在启动......");
    println!("--------");
    println!("命令：'refresh' -> 重新加载所有在相对路径test下的所有markdown文件");
    println!("命令：'reload <path>' -> 重新加载指定路径（<path>）的markdown文件");
    println!("命令：'exit' -> 安全地关闭服务器");
    println!("--------");

    let page_manager = PageManager::init();

    let runner = Runner::init(page_manager);

    Runner::run_server(runner).await;

    println!("bye~");
}
