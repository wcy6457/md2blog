pub mod test;

use crate::test::Test;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use comrak::{markdown_to_html, Options};
use std::fs::read_to_string;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let path = Path::new("test/hello-world.md");
    let test = Arc::new(Mutex::new(Test::new(path, md_file_path_to_html(path))));
    let temp1 = Arc::clone(&test);
    let temp2 = Arc::clone(&test);


    tokio::spawn(commandline_handler(temp2));

    println!("服务器开始监听2233端口，测试页在：localhost:2233/test/hello-world");
    axum::serve(
        match TcpListener::bind("0.0.0.0:2233").await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("发生了错误：{}", e.kind());
                exit(1);
            }
        },
        Router::new().route(
            "/test/hello-world",
            get(move || async move { temp1.lock().await.get_response(StatusCode::OK) }),
        ),
    ).await.unwrap();
}

async fn commandline_handler(temp: Arc<Mutex<Test<'_>>>) {
    loop {
        let mut input = String::new();
        let mut stdin = BufReader::new(io::stdin());

        stdin.read_line(&mut input).await.unwrap_or_else(|e| {
            println!("{}", e);
            0
        });
        let input = input.trim_end();

        // println!("输入了{}", input);
        if input == "reload" {
            println!("reloading......");
            temp.lock().await.update_html();
        }
        if input == "exit" {
            println!("stop~");
            exit(0);
        }
    }
}

fn md_file_path_to_html(path: &Path) -> Result<String, (StatusCode, String)> {
    match read_to_string(path) {
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