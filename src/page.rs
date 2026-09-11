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

#[derive(serde::Deserialize, Clone)]
struct PageConfig {
    uri_path: Option<String>,
}

impl Page {
    pub fn build(file_path: &str, root_path: &str) -> Result<Page, String> {
        let temp_page = Self::new(file_path, root_path);
        if temp_page.uri_path.eq("/admin")
            || temp_page.uri_path.eq("/api")
            || temp_page.uri_path.starts_with("/admin/")
            || temp_page.uri_path.starts_with("/api/")
        {
            return Err(temp_page.uri_path);
        }
        Ok(temp_page)
    }

    /**
    目前视为底层方法，创建page应该通过build方法。


    已经自带错误处理，错误信息会被保存在结构体的html中。


    Page::build_response()已经可以自动按照html中的信息自动构建合适的Response
    */
    fn new(file_path: &str, root_path: &str) -> Page {
        match load_file_by_file_path(file_path, root_path) {
            Ok((page_config, status_code, bytes)) => Page {
                file_path: file_path.to_string(),
                uri_path: match page_config.uri_path {
                    Some(path) => path,
                    None => file_path_into_uri_path(file_path, root_path),
                },
                html: Ok((status_code, bytes)),
            },
            Err((page_config, status_code, message)) => Page {
                file_path: file_path.to_string(),
                uri_path: match page_config.uri_path {
                    Some(path) => path,
                    None => file_path_into_uri_path(file_path, root_path),
                },
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

    pub async fn build_404_response() -> Response {
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
  `Ok((PageConfig, StatusCode::OK, html_bytes))`
  - `PageConfig`： 目前只包含uri_path。
  - `StatusCode::OK`： HTTP 200 状态码。
  - `html_bytes`： 转换后完整 HTML 页面的字节数据。

- 失败时返回：
  `Err((PageConfig, StatusCode::INTERNAL_SERVER_ERROR, error_message))`
  - `PageConfig`： 目前只包含uri_path
  - `StatusCode::INTERNAL_SERVER_ERROR`： HTTP 500 状态码。
  - `error_message`： 读取文件时产生的错误信息。
*/
fn load_file_by_file_path(
    file_path: &str,
    root_path: &str,
) -> Result<(PageConfig, StatusCode, Bytes), (PageConfig, StatusCode, String)> {
    let uri_path = file_path_into_uri_path(file_path, root_path);

    return match read_to_string(Path::new(&file_path)) {
        //能不能读文件？
        Ok(file) => {
            if file.starts_with("<!--") {
                //进入就是可能有配置块
                match file.find("-->") {
                    Some(index) => {
                        //有配置块
                        let yaml = &file[4..index];
                        let html = build_html_bytes(&file[(index + 3)..]);
                        match rust_yaml::serde_integration::from_str::<PageConfig>(yaml) {
                            Ok(yaml) => Ok((yaml, StatusCode::OK, html)),
                            Err(e) => {
                                //配置错误
                                eprintln!(
                                    "加载 {file_path} 中的配置出错。\
                                 将挂载为内部服务器错误。\
                                 错误原因： {e}"
                                );
                                Err((
                                    PageConfig {
                                        uri_path: Some(uri_path),
                                    },
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    e.to_string(),
                                ))
                            }
                        }
                    }
                    None => {
                        //无配置块，返回和file_path相同的路径
                        eprintln!(
                            "在 {file_path} 中发现未闭合的html注释内容！将挂载为内部服务器错误。"
                        );
                        Err((
                            PageConfig {
                                uri_path: Some(uri_path),
                            },
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "未闭合的html注释内容！".to_string(),
                        ))
                    }
                }
            } else {
                //无配置块，返回和file_path相同的路径
                eprintln!(
                    "在 {file_path} 中找不到关于 uri_path 的设置。\
                                 已经回退到默认的按文件路径挂载。"
                );
                Ok((
                    PageConfig {
                        uri_path: Some(uri_path),
                    },
                    StatusCode::OK,
                    build_html_bytes(&file),
                ))
            }
        }
        Err(error) => {
            eprintln!("在读取文件 {file_path} 时发生了错误：{error}");

            Err((
                PageConfig {
                    uri_path: Some(uri_path),
                },
                StatusCode::INTERNAL_SERVER_ERROR,
                error.to_string(),
            ))
        }
    };

    fn build_html_bytes(html: &str) -> Bytes {
        Bytes::from(format!(
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
            markdown_to_html(html, &Options::default())
        ))
    }
}

fn file_path_into_uri_path(file_path: &str, root_path: &str) -> String {
    file_path
        .trim_end_matches(".md")
        .trim_start_matches(root_path)
        .to_string()
}
