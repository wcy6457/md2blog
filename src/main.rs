pub mod test;
pub mod manager;

use crate::manager::FileManager;
use axum::http::StatusCode;
use comrak::{markdown_to_html, Options};
use std::fs::read_to_string;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    // let path = Path::new("test/hello-world.md");
    // let test = Arc::new(Mutex::new(Test::new(path, md_file_path_to_html(path))));
    // // let temp1 = Arc::clone(&test);
    // // let temp2 = Arc::clone(&test);

    let file_manager = FileManager::init().await;
    // let tcp_listener = match TcpListener::bind("0.0.0.0:2233").await {
    //     Ok(l) => l,
    //     Err(e) => {
    //         eprintln!("发生了错误：{}", e.kind());
    //         exit(1);
    //     }
    // };


    tokio::spawn(commandline_handler(Arc::clone(&file_manager)));
    FileManager::run_server(Arc::clone(&file_manager)).await

    // axum::serve(
    //     match TcpListener::bind("0.0.0.0:2233").await {
    //         Ok(l) => l,
    //         Err(e) => {
    //             eprintln!("发生了错误：{}", e.kind());
    //             exit(1);
    //         }
    //     },
    //     Router::new().route(
    //         "/test/hello-world",
    //         get(move || async move { temp1.lock().await.build_response(StatusCode::OK) }),
    //     ),
    // ).await.unwrap();
}

async fn commandline_handler(file_manager: Arc<Mutex<FileManager>>) {
    let mut input = String::new();
    loop {
        let mut stdin = BufReader::new(io::stdin());
        stdin.read_line(&mut input).await.unwrap_or_else(|e| {
            println!("{}", e);
            0
        });
        let temp = input.trim_end();

        // println!("输入了{}", input);
        if temp.starts_with("reload ") {
            let temp = match temp.strip_prefix("reload ") {
                Some(temp) => {
                    temp
                }
                None => {
                    temp
                }
            };
            Arc::clone(&file_manager).lock().await.update_html(temp.trim()).await;
        } else if temp == "refresh" {
            Arc::clone(&file_manager).lock().await.refresh().await;
        } else if temp == "exit" {
            println!("stop~");
            exit(0);
        } else {
            println!("杂鱼，这点指令都输不对~");
        }
        input.clear();
    }
}

fn md_file_path_to_html(path: Arc<String>) -> Result<String, (StatusCode, String)> {
    match read_to_string(Path::new(&*path)) {
        Ok(s) => Ok(format!(
            r#"<!doctype html><html lang="zh-CN"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1"><link rel="stylesheet" href="/test/style.css"></head><body><main class="markdown-body">{}</main></body></html>"#,
            markdown_to_html(s.as_str(), &Options::default())
        )),
        Err(e) => {
            eprintln!("在读取文件的时候发生了错误：{}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}
