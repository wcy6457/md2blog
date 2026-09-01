use axum::body::Body;
use axum::body::Bytes;
use axum::http::StatusCode;
use axum::response::Response;
use comrak::{markdown_to_html, Options};
use std::fs::read_to_string;
use std::path::Path;

pub struct Page {
    pub file_path: String,
    pub uri_path: String,
    pub html: Result<(StatusCode, Bytes), (StatusCode, String)>,
}
impl Page {
    pub fn new(file_path: String) -> Page {
        Page {
            html: md_file_path_to_html_to_bytes(&file_path),
            uri_path: read_uri_path_from_file_path(&file_path),
            file_path,
        }
    }

    // pub fn update_html(&mut self) {
    //     //self.html = md_file_path_to_html_to_bytes(self.path);
    //     match md_file_path_to_html_to_bytes(&self.file_path) {
    //         Ok(html) => {
    //             self.html = Ok(html);
    //             println!("reloading completed!");
    //         }
    //         Err((_, err)) => {
    //             eprintln!("In reload: rebuild html: {}", err);
    //             eprintln!("In reload: nothing changed");
    //         }
    //     }
    // }

    /**
    status_code - 状态码，诸如404，500，200一类

    html - Result<T,Y><br>
    &nbsp;&nbsp;&nbsp;&nbsp;T： html内容<br>
    &nbsp;&nbsp;&nbsp;&nbsp;Y：StatusCode<p>
    调用中若产生错误会直接返回500的错误响应
    */
    pub async fn build_response(&self) -> Response {
        match &self.html {
            Ok((status_code, s)) => Response::builder()
                .status(status_code)
                .header("content-type", "text/html; charset=utf-8")
                .body(Body::from(s.clone()))
                .unwrap_or(Self::build_500_response()),
            Err((code, reason)) => Response::builder()
                .status(code)
                .header("content-type", "text/html; charset=utf-8")
                .body(Body::from(reason.to_string()))
                .unwrap_or(Self::build_500_response()),
        }
    }

    fn build_500_response() -> Response {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("content-type", "text/html; charset=utf-8")
            .body(Body::from(r#"<!doctype html><html lang="zh-CN"><head><meta charset="UTF-8"></head><body><h3>500_未知的服务器错误</h3></body></html>"#))
            .unwrap()
    }

    pub fn build_404_response() -> Response {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "text/html; charset=utf-8")
            .body(Body::from(r#"<!doctype html><html lang="zh-CN"><head><meta charset="UTF-8"></head><body><h3>404_没有这个页面</h3></body></html>"#))
            .unwrap()
    }
}

fn md_file_path_to_html_to_bytes(path: &str) -> Result<(StatusCode, Bytes), (StatusCode, String)> {
    match read_to_string(Path::new(path)) {
        Ok(s) => Ok((
            StatusCode::OK,
            Bytes::from(format!(
                r#"<!doctype html><html lang="zh-CN"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1"><link rel="stylesheet" href="/test/style.css"></head><body><main class="markdown-body">{}</main></body></html>"#,
                markdown_to_html(s.as_str(), &Options::default())
            )),
        )),
        Err(e) => {
            eprintln!("在读取文件的时候发生了错误：{}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

fn read_uri_path_from_file_path(file_path: &String) -> String {
    match read_to_string(Path::new(file_path)) {
        Ok(s) => {
            match s.lines().filter(|line| { line.contains("uri_path:") }).collect::<Vec<&str>>().first() {
                Some(s) => {
                    s.trim().trim_start_matches("uri_path:").to_string()
                }
                None => {
                    eprintln!("在 {file_path} 中找不到关于uri_path的设置。已经回退到默认的按文件路径挂载。");
                    let file_path = file_path.trim_end_matches(".md");
                    format!("/{}", file_path).to_string()
                }
            }
        }
        Err(e) => {
            eprintln!("在加载 {} 的uri_path时发生错误：{}。已经回退到默认的按文件路径挂载。", file_path, e);
            let file_path = file_path.trim_end_matches(".md");
            format!("/{}", file_path).to_string()
        }
    }
}
