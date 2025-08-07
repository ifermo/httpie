//! HTTP客户端模块
//!
//! 负责执行HTTP请求和格式化响应输出。

use crate::error::Result;
use crate::models::HttpRequest;
use reqwest::Client;
use serde_json;

/// HTTP客户端
#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    formatter: ResponseFormatter,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
            formatter: ResponseFormatter::new(),
        }
    }
}

impl HttpClient {
    /// 创建新的HTTP客户端
    pub fn new() -> Self {
        Self::default()
    }

    /// 执行HTTP请求
    pub async fn execute(&self, request: &HttpRequest) -> Result<()> {
        let mut req_builder = self.client.request(request.method.clone(), &request.url);

        // 添加请求头
        for (key, value) in &request.headers {
            req_builder = req_builder.header(key, value);
        }

        // 添加请求体
        if let Some(body) = &request.body {
            req_builder = req_builder.body(body.clone());
        }

        // 发送请求
        let response = req_builder.send().await?;

        // 格式化并打印响应
        self.formatter
            .format_response(&request.name, response)
            .await?;

        Ok(())
    }
}

/// 响应格式化器
#[derive(Debug)]
pub struct ResponseFormatter;

impl ResponseFormatter {
    /// 创建新的响应格式化器
    pub fn new() -> Self {
        Self
    }

    /// 格式化并打印HTTP响应
    pub async fn format_response(
        &self,
        request_name: &str,
        response: reqwest::Response,
    ) -> Result<()> {
        // 打印测试用例名称
        println!("=== {request_name} ===");

        // 打印状态行
        println!(
            "Status: {} {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        );

        // 打印响应头
        if !response.headers().is_empty() {
            println!("Headers:");
            for (name, value) in response.headers() {
                println!("  {}: \"{}\"", name, value.to_str().unwrap_or("<invalid>"));
            }
        }

        // 获取响应体
        let body = response.text().await?;

        // 打印Body标题和内容
        println!("Body:");
        self.format_body(&body);
        println!(); // 结尾空行

        Ok(())
    }

    /// 格式化响应体
    fn format_body(&self, body: &str) {
        if body.trim().is_empty() {
            return;
        }

        // 尝试格式化JSON
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body) {
            if let Ok(pretty_json) = serde_json::to_string_pretty(&json_value) {
                println!("{pretty_json}");
                return;
            }
        }

        // 如果不是JSON，直接打印
        println!("{body}");
    }
}

impl Default for ResponseFormatter {
    fn default() -> Self {
        Self::new()
    }
}
