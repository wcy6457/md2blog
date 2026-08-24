use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use comrak::{markdown_to_html, Options};
use std::fs::read_to_string;
use std::path::Path;
use std::process::exit;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let md = Path::new("test/hello-world.md");
    let md = match read_to_string(md) {
        Ok(s) => {
            Ok(format!(
                r#"<!doctype html><html lang="zh-CN"><head><meta charset="UTF-8"></head><body>{}</body></html>"#,
                markdown_to_html(s.as_str(), &Options::default()
                )
            ))
        }
        Err(e) => {
            eprintln!("在读取文件的时候发生了错误：{}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    };


    // let test = Router::new().route("/test/hello-world", get(|| async { MdFilePath { md } }));
    let test = Router::new().route("/test/hello-world", get(|| async { full_html_to_response(StatusCode::OK, md) }));

    let listener = match TcpListener::bind("0.0.0.0:2233").await {
        Ok(l) => {
            l
        }
        Err(e) => {
            println!("发生了错误：{}", e.kind());
            exit(2);
        }
    };

    println!("服务器开始监听2233端口，测试页在：localhost:2233/test/hello-world");
    axum::serve(listener, test).await.unwrap();
}

/**
status_code - 状态码，诸如404，500，200一类

html - Result<T,Y><br>
&nbsp;&nbsp;&nbsp;&nbsp;T： html内容<br>
&nbsp;&nbsp;&nbsp;&nbsp;Y：StatusCode<p>
调用中若产生错误会直接返回500的错误响应
*/
fn full_html_to_response(hope_status_code: StatusCode, html: Result<String, (StatusCode, String)>) -> Response {
    match html {
        Ok(s) => {
            Response::builder()
                .status(hope_status_code)
                .header("content-type", "text/html; charset=utf-8")
                .body(Body::from(s)).unwrap_or(get_500_response())
        }
        Err((code, reason)) => {
            Response::builder()
                .status(code)
                .header("content-type", "text/html; charset=utf-8")
                .body(Body::from(reason)).unwrap_or(get_500_response())
        }
    }
}

fn get_500_response() -> Response {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("content-type", "text/html; charset=utf-8")
        .body(Body::from(r#"<!doctype html><html lang="zh-CN"><head><meta charset="UTF-8"></head><body><h1>500_内部服务器错误</h1></body></html>"#))
        .unwrap()
}

// struct MdFilePath<'a> {
//     md_file_path: &'a Path,
// }
//
// impl<'a> IntoResponse for MdFilePath<'a> {
//     fn into_response(self) -> Response {
//         match read_to_string(self.md_file_path) {
//             Ok(s) => {
//                 full_html_to_response(StatusCode::ACCEPTED, markdown_to_html(s.as_str(), &Options::default()))
//             }
//             Err(_) => {
//                 get_500_response()
//             }
//         }
//     }
// }




