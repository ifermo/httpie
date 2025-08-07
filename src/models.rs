//! 数据模型模块
//!
//! 定义了HTTP客户端库中使用的核心数据结构。

use crate::DEFAULT_ENVIRONMENT;
use crate::error::{HttpieError, Result};
use reqwest::Method;
use serde_json;
use std::collections::HashMap;
use std::fs;

/// HTTP请求结构体
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub name: String,
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl HttpRequest {
    /// 创建新的HTTP请求
    pub fn new(name: String, method: Method, url: String) -> Self {
        Self {
            name,
            method,
            url,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// 设置请求头
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    /// 设置请求体
    pub fn with_body(mut self, body: Option<String>) -> Self {
        self.body = body;
        self
    }
}

/// 环境变量管理结构体
#[derive(Debug, Default, Clone)]
pub struct Environment {
    variables: HashMap<String, String>,
}

impl Environment {
    /// 创建新的环境实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 从文件加载环境配置
    pub fn from_file(file_path: &str) -> Result<Self> {
        let content = fs::read_to_string(file_path)
            .map_err(|_| HttpieError::FileNotFound(file_path.to_string()))?;

        let env_data: HashMap<String, HashMap<String, String>> = serde_json::from_str(&content)?;

        let variables = env_data
            .get(DEFAULT_ENVIRONMENT)
            .cloned()
            .unwrap_or_default();

        Ok(Self { variables })
    }

    /// 获取变量值
    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// 插入变量
    pub fn insert(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    /// 扩展变量集合
    pub fn extend(&mut self, other: HashMap<String, String>) {
        self.variables.extend(other);
    }

    /// 获取所有变量的引用
    pub fn variables(&self) -> &HashMap<String, String> {
        &self.variables
    }
}
