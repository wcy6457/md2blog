pub mod command;
pub mod manager;
pub mod runner;
pub mod test;

use crate::manager::FileManager;
use crate::runner::Runner;
#[tokio::main]
async fn main() {
    println!("Server has been started.");

    let file_manager = FileManager::init();

    let runner = Runner::init(file_manager);

    Runner::run_server(runner).await;

    println!("bye~");
}
