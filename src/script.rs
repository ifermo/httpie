//! 响应处理器脚本模块
//!
//! 实现基于deno_core的JavaScript脚本执行引擎，支持响应处理和测试断言。

use crate::error::{HttpieError, Result};
use deno_core::{JsRuntime, RuntimeOptions};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

/// 脚本执行引擎
pub struct ScriptEngine {
    runtime: JsRuntime,
    global_variables: HashMap<String, Value>,
}

/// 响应对象，用于在JavaScript中访问HTTP响应信息
#[derive(Debug, Clone)]
pub struct ResponseObject {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Value,
    pub content_type: String,
}

/// 测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
}

impl ScriptEngine {
    /// 创建新的脚本执行引擎
    pub fn new() -> Result<Self> {
        let runtime = JsRuntime::new(RuntimeOptions::default());

        Ok(Self {
            runtime,
            global_variables: HashMap::new(),
        })
    }

    /// 执行响应处理脚本
    pub async fn execute_response_script(
        &mut self,
        script: String,
        response_obj: ResponseObject,
    ) -> Result<Vec<TestResult>> {
        // 初始化JavaScript环境
        self.setup_javascript_environment(&response_obj)?;

        // 执行脚本
        let result = self.runtime.execute_script("<response_handler>", script);

        match result {
            Ok(_) => {
                // 提取测试结果
                self.extract_test_results()
            }
            Err(e) => Err(HttpieError::ScriptError(format!(
                "Script execution failed: {}",
                e
            ))),
        }
    }

    /// 设置JavaScript环境
    fn setup_javascript_environment(&mut self, response_obj: &ResponseObject) -> Result<()> {
        // 注入response对象
        let response_json = json!({
            "status": response_obj.status,
            "headers": response_obj.headers,
            "body": response_obj.body,
            "contentType": response_obj.content_type
        });

        let setup_script = format!(
            r#"
            // 全局变量存储
            globalThis.__httpie_globals = globalThis.__httpie_globals || {{}};
            globalThis.__httpie_tests = [];

            // 响应对象
            globalThis.response = {};

            // 客户端对象
            globalThis.client = {{
                global: {{
                    set: function(key, value) {{
                        globalThis.__httpie_globals[key] = value;
                    }},
                    get: function(key) {{
                        return globalThis.__httpie_globals[key];
                    }}
                }},
                test: function(name, testFn) {{
                    try {{
                        testFn();
                        globalThis.__httpie_tests.push({{
                            name: name,
                            passed: true,
                            message: null
                        }});
                    }} catch (error) {{
                        globalThis.__httpie_tests.push({{
                            name: name,
                            passed: false,
                            message: error.message
                        }});
                    }}
                }},
                assert: function(condition, message) {{
                    if (!condition) {{
                        throw new Error(message || 'Assertion failed');
                    }}
                }}
            }};

            // 控制台对象
            globalThis.console = {{
                log: function(...args) {{
                    // 简单的日志输出，实际项目中可以改进
                }}
            }};

            // 全局assert函数
            globalThis.assert = function(condition, message) {{
                if (!condition) {{
                    throw new Error(message || 'Assertion failed');
                }}
            }};
            "#,
            serde_json::to_string(&response_json).unwrap()
        );

        self.runtime
            .execute_script("<setup>", setup_script)
            .map_err(|e| HttpieError::ScriptError(format!("Failed to setup environment: {}", e)))?;

        Ok(())
    }

    /// 提取测试结果
    fn extract_test_results(&mut self) -> Result<Vec<TestResult>> {
        let extract_script = r#"
            JSON.stringify(globalThis.__httpie_tests || []);
        "#;

        let result = self
            .runtime
            .execute_script("<extract_tests>", extract_script)
            .map_err(|e| {
                HttpieError::ScriptError(format!("Failed to extract test results: {}", e))
            })?;

        let global = result.open(&mut self.runtime.handle_scope());
        let result_str = global.to_rust_string_lossy(&mut self.runtime.handle_scope());

        let test_results: Vec<TestResult> = serde_json::from_str(&result_str).map_err(|e| {
            HttpieError::ScriptError(format!("Failed to parse test results: {}", e))
        })?;

        // 提取全局变量
        self.extract_global_variables()?;

        Ok(test_results)
    }

    /// 提取全局变量
    fn extract_global_variables(&mut self) -> Result<()> {
        let extract_script = r#"
            JSON.stringify(globalThis.__httpie_globals || {});
        "#;

        let result = self
            .runtime
            .execute_script("<extract_globals>", extract_script)
            .map_err(|e| {
                HttpieError::ScriptError(format!("Failed to extract global variables: {}", e))
            })?;

        let global = result.open(&mut self.runtime.handle_scope());
        let result_str = global.to_rust_string_lossy(&mut self.runtime.handle_scope());

        let globals: HashMap<String, Value> = serde_json::from_str(&result_str).map_err(|e| {
            HttpieError::ScriptError(format!("Failed to parse global variables: {}", e))
        })?;

        self.global_variables.extend(globals);
        Ok(())
    }

    /// 获取全局变量
    pub fn get_global_variable(&self, key: &str) -> Option<&Value> {
        self.global_variables.get(key)
    }

    /// 获取所有全局变量
    pub fn get_all_global_variables(&self) -> &HashMap<String, Value> {
        &self.global_variables
    }
}

impl ResponseObject {
    /// 从reqwest::Response创建ResponseObject
    pub async fn from_response(response: Response) -> Result<Self> {
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let mut headers = HashMap::new();
        for (name, value) in response.headers() {
            headers.insert(name.to_string(), value.to_str().unwrap_or("").to_string());
        }

        let body_text = response.text().await?;
        let body = if content_type.contains("application/json") {
            serde_json::from_str(&body_text).unwrap_or(Value::String(body_text))
        } else {
            Value::String(body_text)
        };

        Ok(Self {
            status,
            headers,
            body,
            content_type,
        })
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create script engine")
    }
}
