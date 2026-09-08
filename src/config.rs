use std::fs::read_to_string;
use std::path::Path;
use std::process::exit;

#[derive(serde::Deserialize, Clone)]
pub struct Config {
    content_root: String,
    port: String,
}

impl Config {
    pub fn get_root(&self) -> &str {
        &self.content_root
    }

    pub fn get_port(&self) -> &str {
        &self.port
    }

    pub fn init() -> Config {
        match std::env::current_exe() {
            Ok(path) => {
                let path = match path.as_path().parent() {
                    Some(path) => path,
                    None => {
                        eprintln!("无法读取运行路径。");
                        exit(1);
                    }
                };
                let path = path.join("md2blog.yaml");
                if !path.exists() {
                    eprintln!("配置文件md2blog.yaml不存在！");
                    exit(1);
                };

                match read_to_string(path) {
                    Ok(result) => match rust_yaml::serde_integration::from_str::<Config>(&result) {
                        Ok(mut config) => {
                            let content_root = Path::new(config.get_root());
                            if !content_root.is_absolute() {
                                eprintln!("根目录必须为绝对路径！");
                                exit(1);
                            }

                            if !content_root.is_dir() {
                                eprintln!("根目录不存在或不是目录：{}", content_root.display());
                                exit(1);
                            }

                            config.content_root = match content_root.canonicalize() {
                                Ok(path) => path.to_string_lossy().replace('\\', "/"),
                                Err(e) => {
                                    eprintln!("无法规范化根目录：{e}");
                                    exit(1);
                                }
                            };
                            config
                        }
                        Err(e) => {
                            eprintln!("加载配置文件时出错：{e}");
                            exit(1);
                        }
                    },
                    Err(e) => {
                        eprintln!("无法读取配置文件！{e}");
                        exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("加载配置文件时出错：获取当前路径时发生错误：{e}");
                exit(1);
            }
        }
    }
}
