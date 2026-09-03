use axum::body::Body;
use axum::body::Bytes;
use axum::http::StatusCode;
use axum::response::Response;
use comrak::{Options, markdown_to_html};
use std::fs::read_to_string;
use std::path::Path;

/**
Page::new()已经自带错误处理，错误信息会被保存在结构体的html中。


Page::build_response()已经可以自动按照html中的信息自动构建合适的Response
*/
pub struct Page {
    pub file_path: String,
    pub uri_path: String,
    pub html: Result<(StatusCode, Bytes), (StatusCode, String)>,
}
impl Page {
    /**
    已经自带错误处理，错误信息会被保存在结构体的html中。


    Page::build_response()已经可以自动按照html中的信息自动构建合适的Response
    */
    pub fn new(file_path: &str) -> Page {
        match load_file_by_file_path(file_path) {
            Ok((uri_path, status_code, bytes)) => Page {
                file_path: file_path.to_string(),
                uri_path,
                html: Ok((status_code, bytes)),
            },
            Err((status_code, message)) => Page {
                file_path: file_path.to_string(),
                uri_path: file_path.to_string(),
                html: Err((status_code, message)),
            },
        }
    }

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

/**
读取 Markdown 文件，并生成该页面的 URI 路径和 HTML 内容。

# 返回值

- 成功时返回：
  `Ok((uri_path, StatusCode::OK, html_bytes))`
  - `uri_path`：Markdown 中配置的访问路径；未配置时根据文件路径生成。
  - `StatusCode::OK`：HTTP 200 状态码。
  - `html_bytes`：转换后完整 HTML 页面的字节数据。

- 失败时返回：
  `Err((StatusCode::INTERNAL_SERVER_ERROR, error_message))`
  - `StatusCode::INTERNAL_SERVER_ERROR`：HTTP 500 状态码。
  - `error_message`：读取文件时产生的错误信息。
*/
fn load_file_by_file_path(
    file_path: &str,
) -> Result<(String, StatusCode, Bytes), (StatusCode, String)> {
    match read_to_string(Path::new(file_path)) {
        Ok(markdown) => {
            let uri_path = markdown
                .lines()
                .find(|line| line.contains("uri_path:"))
                .map(|line| {
                    line.trim()
                        .trim_start_matches("uri_path:")
                        .trim()
                        .to_string()
                })
                .unwrap_or_else(|| {
                    eprintln!(
                        "在 {file_path} 中找不到关于 uri_path 的设置。\
                        已经回退到默认的按文件路径挂载。"
                    );

                    let path_without_extension = file_path.trim_end_matches(".md");

                    format!("/{path_without_extension}")
                });

            let html = format!(
                r#"<!doctype html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="stylesheet" href="/test/style.css">
</head>
<body>
    <main class="markdown-body">{}</main>
</body>
</html>"#,
                markdown_to_html(&markdown, &Options::default())
            );

            Ok((uri_path, StatusCode::OK, Bytes::from(html)))
        }

        Err(error) => {
            eprintln!("在读取文件 {file_path} 时发生了错误：{error}");

            Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
        }
    }
}
