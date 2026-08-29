use crate::utils::md_file_path_to_html;
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use std::sync::Arc;

pub struct Test {
    path: Arc<String>,
    html: Result<String, (StatusCode, String)>,
}
impl Test {
    pub fn new(path: Arc<String>, html: Result<String, (StatusCode, String)>) -> Test {
        Test {
            path,
            html,
        }
    }

    pub fn update_html(&mut self) {
        //self.html = md_file_path_to_html(self.path);
        match md_file_path_to_html(self.path.clone()) {
            Ok(html) => {
                self.html = Ok(html);
                println!("reloading completed!");
            }
            Err((_, err)) => {
                eprintln!("In reload: rebuild html: {}", err);
            }
        }
    }

    /**
    status_code - 状态码，诸如404，500，200一类

    html - Result<T,Y><br>
    &nbsp;&nbsp;&nbsp;&nbsp;T： html内容<br>
    &nbsp;&nbsp;&nbsp;&nbsp;Y：StatusCode<p>
    调用中若产生错误会直接返回500的错误响应
    */
    pub fn build_response(&self, hope_status_code: StatusCode) -> Response {
        match &self.html {
            Ok(s) => {
                Response::builder()
                    .status(hope_status_code)
                    .header("content-type", "text/html; charset=utf-8")
                    .body(Body::from(s.to_string())).unwrap_or(Self::build_500_response())
            }
            Err((code, reason)) => {
                Response::builder()
                    .status(code)
                    .header("content-type", "text/html; charset=utf-8")
                    .body(Body::from(reason.to_string())).unwrap_or(Self::build_500_response())
            }
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