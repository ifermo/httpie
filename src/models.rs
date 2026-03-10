//! 数据模型模块
//!
//! 定义了HTTP客户端库中使用的核心数据结构。

use crate::DEFAULT_ENVIRONMENT;
use crate::error::{HttpieError, Result};
use reqwest::Method;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;

/// HTTP请求结构体
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub name: String,
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub response_handler: Option<String>,
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
            response_handler: None,
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

    /// 设置响应处理器脚本
    pub fn with_response_handler(mut self, response_handler: Option<String>) -> Self {
        self.response_handler = response_handler;
        self
    }
}

/// 环境变量管理结构体
#[derive(Debug, Default, Clone)]
pub struct Environment {
    variables: HashMap<String, String>,
    dns_overrides: HashMap<String, SocketAddr>,
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

        let env_data: serde_json::Value = serde_json::from_str(&content)?;

        let mut variables = HashMap::new();
        let mut dns_overrides = HashMap::new();

        let Some(env_obj) = env_data
            .get(DEFAULT_ENVIRONMENT)
            .and_then(|v| v.as_object())
        else {
            return Ok(Self {
                variables,
                dns_overrides,
            });
        };

        for (key, value) in env_obj {
            if key == "dns" {
                if let Some(dns_obj) = value.as_object() {
                    for (domain, addr_value) in dns_obj {
                        let Some(addr_str) = addr_value.as_str() else {
                            continue;
                        };
                        let addr: SocketAddr = addr_str.parse().map_err(|e| {
                            HttpieError::Parse(format!(
                                "Invalid dns override for '{domain}': {addr_str} ({e})"
                            ))
                        })?;
                        dns_overrides.insert(domain.clone(), addr);
                    }
                }
                continue;
            }

            let str_value = match value {
                serde_json::Value::String(s) => Some(s.clone()),
                serde_json::Value::Number(n) => Some(n.to_string()),
                serde_json::Value::Bool(b) => Some(b.to_string()),
                _ => None,
            };

            if let Some(str_value) = str_value {
                variables.insert(key.clone(), str_value);
            }
        }

        Ok(Self {
            variables,
            dns_overrides,
        })
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

    pub fn dns_overrides(&self) -> &HashMap<String, SocketAddr> {
        &self.dns_overrides
    }
}
