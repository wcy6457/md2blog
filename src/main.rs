use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use comrak::{markdown_to_html, Options};
use std::fs::read_to_string;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use tokio::net::TcpListener;


struct Test<'a> {
    path: &'a Path,
    html: Result<String, (StatusCode, String)>,
}
impl<'a> Test<'a> {
    pub async fn run(&self, listener: TcpListener, router: Router) {
        println!("服务器开始监听2233端口，测试页在：localhost:2233/test/hello-world");
        axum::serve(listener, router).await.unwrap();
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
impl<'a> Clone for Test<'a> {
    fn clone(&self) -> Self {
        Test {
            path: self.path.clone(),
            html: self.html.clone(),
        }
    }
}

#[tokio::main]
async fn main() {
    let path = Path::new("test/hello-world.md");
    let test = Arc::new(Test {
        path,
        html: md_file_path_to_html(path),
    });
    let temp = Arc::clone(&test);

    // let test = Router::new().route("/test/hello-world", get(|| async { MdFilePath { md } }));
    let router = Router::new().route("/test/hello-world", get(move ||
        {
            async move { temp.build_response(StatusCode::OK) }
        }
    ));

    let listener = match TcpListener::bind("0.0.0.0:2233").await {
        Ok(l) => {
            l
        }
        Err(e) => {
            println!("发生了错误：{}", e.kind());
            exit(2);
        }
    };

    test.run(listener, router).await;
}


async fn commandline_handler(path: &Path, md: &mut Result<String, (StatusCode, String)>) {
    todo!()
    // loop {
    //     let mut input = String::new();
    //     let mut stdin = BufReader::new(io::stdin());
    //
    //     stdin.read_line(&mut input).await.unwrap_or_else(|e| {
    //         println!("{}", e);
    //         0
    //     });
    // }
}

fn md_file_path_to_html(path: &Path) -> Result<String, (StatusCode, String)> {
    match read_to_string(path) {
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
    }
}


// struct MdFilePath<'a> {
//     md_file_path: &'a Path,
// }
//
// impl<'a> IntoResponse for MdFilePath<'a> {
//     fn into_response(self) -> Response {
//         match read_to_string(self.md_file_path) {
//             Ok(s) => {
//                 build_response(StatusCode::ACCEPTED, markdown_to_html(s.as_str(), &Options::default()))
//             }
//             Err(_) => {
//                 get_500_response()
//             }
//         }
//     }
// }




