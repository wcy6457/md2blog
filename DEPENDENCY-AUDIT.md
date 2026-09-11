# 依赖许可证清单

依据 Cargo.lock，包含直接、传递、构建及平台专用依赖。上游许可可选 MIT 时，本项目选择 MIT；AND 连接的许可仍须同时遵守。

| 依赖                  | 版本                          | 上游许可声明                                        | 本项目采用                      |
|-----------------------|-------------------------------|-----------------------------------------------------|---------------------------------|
| aho-corasick          | 1.1.5                         | Unlicense OR MIT                                    | MIT                             |
| arc-swap              | 1.9.2                         | MIT OR Apache-2.0                                   | MIT                             |
| atomic-waker          | 1.1.2                         | Apache-2.0 OR MIT                                   | MIT                             |
| axum                  | 0.8.9                         | MIT                                                 | MIT                             |
| axum-core             | 0.5.6                         | MIT                                                 | MIT                             |
| base64                | 0.22.1                        | MIT OR Apache-2.0                                   | MIT                             |
| bytes                 | 1.12.1                        | MIT                                                 | MIT                             |
| caseless              | 0.2.2                         | MIT                                                 | MIT                             |
| comrak                | 0.55.0                        | BSD-2-Clause                                        | BSD-2-Clause，另见下方说明      |
| entities              | 1.0.1                         | MIT                                                 | MIT                             |
| equivalent            | 1.0.2                         | Apache-2.0 OR MIT                                   | MIT                             |
| fastrand              | 2.5.0                         | Apache-2.0 OR MIT                                   | MIT                             |
| finl_unicode          | 1.4.0                         | (MIT OR Apache-2.0) AND Unicode-DFS-2016            | MIT；Unicode 许可见下方差异说明 |
| futures-channel       | 0.3.34                        | MIT OR Apache-2.0                                   | MIT                             |
| futures-core          | 0.3.34                        | MIT OR Apache-2.0                                   | MIT                             |
| futures-task          | 0.3.34                        | MIT OR Apache-2.0                                   | MIT                             |
| futures-util          | 0.3.34                        | MIT OR Apache-2.0                                   | MIT                             |
| glob                  | 0.3.4                         | MIT OR Apache-2.0                                   | MIT                             |
| hashbrown             | 0.17.1                        | MIT OR Apache-2.0                                   | MIT                             |
| http                  | 1.5.0                         | MIT OR Apache-2.0                                   | MIT                             |
| http-body             | 1.1.0                         | MIT                                                 | MIT                             |
| http-body-util        | 0.1.5                         | MIT                                                 | MIT                             |
| httparse              | 1.10.1                        | MIT OR Apache-2.0                                   | MIT                             |
| httpdate              | 1.0.3                         | MIT OR Apache-2.0                                   | MIT                             |
| hyper                 | 1.11.1                        | MIT                                                 | MIT                             |
| hyper-util            | 0.1.20                        | MIT                                                 | MIT                             |
| indexmap              | 2.14.2                        | Apache-2.0 OR MIT                                   | MIT                             |
| itoa                  | 1.0.18                        | MIT OR Apache-2.0                                   | MIT                             |
| jetscii               | 0.5.3                         | MIT OR Apache-2.0                                   | MIT                             |
| libc                  | 0.2.189                       | MIT OR Apache-2.0                                   | MIT                             |
| matchit               | 0.8.4                         | MIT AND BSD-3-Clause                                | MIT AND BSD-3-Clause            |
| memchr                | 2.8.3                         | Unlicense OR MIT                                    | MIT                             |
| mime                  | 0.3.17                        | MIT OR Apache-2.0                                   | MIT                             |
| mio                   | 1.2.3                         | MIT                                                 | MIT                             |
| percent-encoding      | 2.3.2                         | MIT OR Apache-2.0                                   | MIT                             |
| phf                   | 0.13.1                        | MIT                                                 | MIT                             |
| phf_codegen           | 0.13.1                        | MIT                                                 | MIT                             |
| phf_generator         | 0.13.1                        | MIT                                                 | MIT                             |
| phf_shared            | 0.13.1                        | MIT                                                 | MIT                             |
| pin-project-lite      | 0.2.17                        | Apache-2.0 OR MIT                                   | MIT                             |
| proc-macro2           | 1.0.107                       | MIT OR Apache-2.0                                   | MIT                             |
| quote                 | 1.0.47                        | MIT OR Apache-2.0                                   | MIT                             |
| regex                 | 1.13.1                        | MIT OR Apache-2.0                                   | MIT                             |
| regex-automata        | 0.4.18                        | MIT OR Apache-2.0                                   | MIT                             |
| regex-syntax          | 0.8.11                        | MIT OR Apache-2.0                                   | MIT                             |
| rust-yaml             | 1.1.0                         | MIT OR Apache-2.0                                   | MIT                             |
| rustc-hash            | 2.1.3                         | Apache-2.0 OR MIT                                   | MIT                             |
| rustversion           | 1.0.23                        | MIT OR Apache-2.0                                   | MIT                             |
| serde                 | 1.0.229                       | MIT OR Apache-2.0                                   | MIT                             |
| serde_core            | 1.0.229                       | MIT OR Apache-2.0                                   | MIT                             |
| serde_derive          | 1.0.229                       | MIT OR Apache-2.0                                   | MIT                             |
| siphasher             | 1.0.3                         | MIT/Apache-2.0                                      | MIT                             |
| slab                  | 0.4.12                        | MIT                                                 | MIT                             |
| smallvec              | 1.16.0                        | MIT OR Apache-2.0                                   | MIT                             |
| socket2               | 0.6.5                         | MIT OR Apache-2.0                                   | MIT                             |
| syn                   | 3.0.5                         | MIT OR Apache-2.0                                   | MIT                             |
| sync_wrapper          | 1.0.2                         | Apache-2.0                                          | Apache-2.0                      |
| tinyvec               | 1.13.2                        | Zlib OR Apache-2.0 OR MIT                           | MIT                             |
| tinyvec_macros        | 0.1.1                         | MIT OR Apache-2.0 OR Zlib                           | MIT                             |
| tokio                 | 1.53.1                        | MIT                                                 | MIT                             |
| tokio-macros          | 2.7.2                         | MIT                                                 | MIT                             |
| tower                 | 0.5.3                         | MIT                                                 | MIT                             |
| tower-layer           | 0.3.3                         | MIT                                                 | MIT                             |
| tower-service         | 0.3.3                         | MIT                                                 | MIT                             |
| typed-arena           | 2.0.2                         | MIT                                                 | MIT                             |
| unicode-ident         | 1.0.24                        | (MIT OR Apache-2.0) AND Unicode-3.0                 | MIT AND Unicode-3.0             |
| unicode-normalization | 0.1.25                        | MIT OR Apache-2.0                                   | MIT                             |
| wasi                  | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | MIT                             |
| windows-link          | 0.2.1                         | MIT OR Apache-2.0                                   | MIT                             |
| windows-sys           | 0.61.2                        | MIT OR Apache-2.0                                   | MIT                             |

## 随包许可说明

- comrak：COPYING 还包含继承代码的 MIT/BSD 声明；其中 CC-BY-SA-4.0 指向 CommonMark 测试规范，不代表库代码整体的许可证。
- matchit：同时保留 LICENSE（MIT）和 LICENSE.httprouter（BSD-3-Clause）。
- atomic-waker：还包含 LICENSE-THIRD-PARTY，其中相关代码提供 MIT / Apache-2.0 许可文本，本项目采用 MIT。
- siphasher：随包 COPYING 明确说明 MIT/Apache-2.0 可二选一，本项目采用 MIT。
- unicode-ident：除 MIT 外，还须保留 LICENSE-UNICODE（Unicode-3.0）。
- finl_unicode：元数据声明 Unicode-DFS-2016，随包 LICENSE-UNICODE 却是 Unicode License V3；差异尚待上游确认，保留随包
  Unicode 原文，不能省略数据许可。

本清单仅记录许可选择。
