//! HTTP客户端库
//!
//! 这是一个功能完整的HTTP客户端库，支持解析.http文件格式，
//! 变量替换，环境配置等功能。

pub mod client;
pub mod environment;
pub mod error;
pub mod models;
pub mod parser;
pub mod script;
pub mod variable;

// 重新导出主要的公共API
pub use client::{HttpClient, ResponseFormatter};
pub use environment::EnvironmentLoader;
pub use error::{HttpieError, Result};
pub use models::{Environment, HttpRequest};
pub use parser::HttpParser;
pub use script::{ResponseObject, ScriptEngine, TestResult};
pub use variable::VariableReplacer;

// 常量定义
pub const DEFAULT_HTTP_FILE: &str = "./test.http";
pub const DEFAULT_ENV_FILE: &str = "httpie.env.json";
pub const DEFAULT_ENVIRONMENT: &str = "development";
pub const SUPPORTED_METHODS: &[&str] = &[
    "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "GRAPHQL",
];
