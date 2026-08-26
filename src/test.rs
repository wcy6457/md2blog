use crate::md_file_path_to_html;
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use std::path::Path;

pub struct Test<'a> {
    path: &'a Path,
    html: Result<String, (StatusCode, String)>,
}
impl<'a> Test<'a> {
    pub fn new(path: &'a Path, html: Result<String, (StatusCode, String)>) -> Test<'a> {
        Test {
            path,
            html,
        }
    }

    pub fn update_html(&mut self) {
        //self.html = md_file_path_to_html(self.path);
        match md_file_path_to_html(self.path) {
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
    pub fn get_response(&self, hope_status_code: StatusCode) -> Response {
        match &self.html {
            Ok(s) => {
                Response::builder()
                    .status(hope_status_code)
                    .header("content-type", "text/html; charset=utf-8")
                    .body(Body::from(s.to_string())).unwrap_or(Self::get_500_response())
            }
            Err((code, reason)) => {
                Response::builder()
                    .status(code)
                    .header("content-type", "text/html; charset=utf-8")
                    .body(Body::from(reason.to_string())).unwrap_or(Self::get_500_response())
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
}