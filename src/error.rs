//! 错误处理模块
//!
//! 定义了HTTP客户端库中使用的所有错误类型。

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpieError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Invalid request format: {0}")]
    InvalidRequest(String),
    #[error("Script execution error: {0}")]
    ScriptError(String),
    #[error("Script parsing error: {0}")]
    ScriptParsingError(String),
}

/// Result类型别名，简化错误处理
pub type Result<T> = std::result::Result<T, HttpieError>;
