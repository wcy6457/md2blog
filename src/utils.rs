use axum::http::StatusCode;
use comrak::{markdown_to_html, Options};
use std::fs::read_to_string;
use std::path::Path;
use std::sync::Arc;

pub fn md_file_path_to_html(path: Arc<String>) -> Result<String, (StatusCode, String)> {
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