pub mod test;
pub mod manager;

use crate::manager::FileManager;
use crate::test::Test;
use axum::http::StatusCode;
use comrak::{markdown_to_html, Options};
use glob::glob;
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

    let file_manager = Arc::new(Mutex::new(FileManager::default()));
    // let tcp_listener = match TcpListener::bind("0.0.0.0:2233").await {
    //     Ok(l) => l,
    //     Err(e) => {
    //         eprintln!("发生了错误：{}", e.kind());
    //         exit(1);
    //     }
    // };


    for entry in glob("test/*.md").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let path = match path.into_string() {
                    Ok(s) => {
                        s
                    }
                    Err(e) => {
                        println!("搜寻文件将路径转换为String时出错：{:?}", e);
                        String::new()
                    }
                };
                let path = Arc::new(path.replace('\\', "/"));
                let path1 = Arc::clone(&path);
                let path2 = Arc::clone(&path);
                let path3 = Arc::clone(&path);
                file_manager.lock().await.add(path1, Arc::new(Mutex::new(Test::new(path2, md_file_path_to_html(path3))))).await;
            }
            Err(e) => {
                println!("{:?}", e);
                exit(1);
            }
        }
    }

    tokio::spawn(commandline_handler(Arc::clone(&file_manager)));
    FileManager::run_server(file_manager).await;

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
    loop {
        let mut input = String::new();
        let mut stdin = BufReader::new(io::stdin());

        stdin.read_line(&mut input).await.unwrap_or_else(|e| {
            println!("{}", e);
            0
        });
        let input = input.trim_end();

        // println!("输入了{}", input);
        if let Some(path) = input.strip_prefix("reload ") {
            println!("reloading......");
            file_manager.lock().await.update_html(path.trim()).await;
        } else {}
        if input == "exit" {
            println!("stop~");
            exit(0);
        }
    }
}

fn md_file_path_to_html(path: Arc<String>) -> Result<String, (StatusCode, String)> {
    match read_to_string(Path::new(&*path)) {
        Ok(s) => Ok(format!(
            r#"<!doctype html><html lang="zh-CN"><head><meta charset="UTF-8"></head><body>{}</body></html>"#,
            markdown_to_html(s.as_str(), &Options::default())
        )),
        Err(e) => {
            eprintln!("在读取文件的时候发生了错误：{}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}
