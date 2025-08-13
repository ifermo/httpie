//! HTTP客户端模块
//!
//! 负责执行HTTP请求和格式化响应输出。

use crate::error::Result;
use crate::models::HttpRequest;
use crate::script::{ResponseObject, ScriptEngine, TestResult};
use reqwest::Client;
use serde_json;

/// HTTP客户端
pub struct HttpClient {
    client: Client,
    formatter: ResponseFormatter,
    script_engine: Option<ScriptEngine>,
    print_response: bool,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
            formatter: ResponseFormatter::new(),
            script_engine: None,
            print_response: true,
        }
    }
}

impl HttpClient {
    /// 创建新的HTTP客户端
    pub fn new() -> Self {
        Self::default()
    }

    /// 启用脚本功能
    pub fn with_script_engine(mut self) -> Result<Self> {
        self.script_engine = Some(ScriptEngine::new()?);
        Ok(self)
    }

    /// 控制是否打印响应（默认打印）
    pub fn with_print_response(mut self, enabled: bool) -> Self {
        self.print_response = enabled;
        self
    }

    /// 执行HTTP请求
    pub async fn execute(&mut self, request: &HttpRequest) -> Result<()> {
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

        // 如果有响应处理器脚本，执行脚本
        if let Some(script) = &request.response_handler {
            if let Some(ref mut engine) = self.script_engine {
                // 创建响应对象
                let response_obj = ResponseObject::from_response(response).await?;

                // 执行脚本
                let test_results = engine
                    .execute_response_script(script.clone(), response_obj.clone())
                    .await?;

                // 打印测试结果
                self.formatter
                    .format_test_results(&request.name, &test_results);

                // 格式化并打印响应（使用克隆的响应对象），受开关控制
                if self.print_response {
                    self.formatter
                        .format_response_from_object(&request.name, &response_obj)
                        .await?;
                }
            } else {
                return Err(crate::error::HttpieError::ScriptError(
                    "Script engine not initialized. Call with_script_engine() first.".to_string(),
                ));
            }
        } else {
            // 没有脚本，直接格式化并打印响应（受开关控制）
            if self.print_response {
                self.formatter
                    .format_response(&request.name, response)
                    .await?;
            }
        }

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
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body)
            && let Ok(pretty_json) = serde_json::to_string_pretty(&json_value)
        {
            println!("{pretty_json}");
            return;
        }

        // 如果不是JSON，直接打印
        println!("{body}");
    }

    /// 格式化测试结果
    pub fn format_test_results(&self, request_name: &str, test_results: &[TestResult]) {
        if !test_results.is_empty() {
            println!("\n=== Test Results for {} ===", request_name);
            for result in test_results {
                let status = if result.passed {
                    "✓ PASS"
                } else {
                    "✗ FAIL"
                };
                println!("{} {}", status, result.name);
                if let Some(message) = &result.message {
                    println!("  Message: {}", message);
                }
            }
            println!();
        }
    }

    /// 从ResponseObject格式化响应
    pub async fn format_response_from_object(
        &self,
        request_name: &str,
        response_obj: &ResponseObject,
    ) -> Result<()> {
        // 打印测试用例名称
        println!("=== {request_name} ===");

        // 打印状态行
        println!("Status: {}", response_obj.status);

        // 打印响应头
        if !response_obj.headers.is_empty() {
            println!("Headers:");
            for (name, value) in &response_obj.headers {
                println!("  {}: \"{}\"", name, value);
            }
        }

        // 打印Body标题和内容
        println!("Body:");
        match &response_obj.body {
            serde_json::Value::String(s) => self.format_body(s),
            other => {
                if let Ok(pretty_json) = serde_json::to_string_pretty(other) {
                    println!("{}", pretty_json);
                } else {
                    println!("{}", other);
                }
            }
        }
        println!(); // 结尾空行

        Ok(())
    }
}

impl Default for ResponseFormatter {
    fn default() -> Self {
        Self::new()
    }
}
