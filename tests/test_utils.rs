//! 测试工具模块
//!
//! 提供测试中使用的辅助函数和常量

use httpie::{Environment, HttpRequest};
use reqwest::Method;
use std::collections::HashMap;

/// 创建测试用的HTTP请求
pub fn create_test_request() -> HttpRequest {
    HttpRequest::new(
        "test_request".to_string(),
        Method::GET,
        "https://httpbin.org/get".to_string(),
    )
}

/// 创建带请求头的测试请求
pub fn create_test_request_with_headers() -> HttpRequest {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());

    HttpRequest::new(
        "test_request_with_headers".to_string(),
        Method::POST,
        "https://httpbin.org/post".to_string(),
    )
    .with_headers(headers)
    .with_body(Some("{\"key\": \"value\"}".to_string()))
}

/// 创建测试环境
pub fn create_test_environment() -> Environment {
    let mut env = Environment::new();
    env.insert(
        "BASE_URL".to_string(),
        "https://api.example.com".to_string(),
    );
    env.insert("API_KEY".to_string(), "test_api_key_123".to_string());
    env.insert("VERSION".to_string(), "v1".to_string());
    env
}

/// 创建测试用的.http文件内容
pub fn create_test_http_content() -> String {
    r#"@baseUrl = https://httpbin.org
@token = test_token_123

### Test GET Request
GET {{baseUrl}}/get
User-Agent: httpie-test
Authorization: Bearer {{token}}

### Test POST Request
POST {{baseUrl}}/post
Content-Type: application/json

{
  "name": "test",
  "value": 123
}

> {
%
client.test("Status should be 200", function() {
    client.assert(response.status === 200, "Expected status 200");
});

client.test("Response should have json content", function() {
    client.assert(response.body.json !== undefined, "Expected JSON response");
});
%
}

### Test PUT Request
PUT {{baseUrl}}/put
Content-Type: application/json

{
  "id": 1,
  "updated": true
}
"#
    .to_string()
}

/// 创建测试用的环境文件内容
pub fn create_test_env_content() -> String {
    r#"{
  "development": {
    "BASE_URL": "https://dev.api.example.com",
    "API_KEY": "dev_key_123",
    "TIMEOUT": "30"
  },
  "production": {
    "BASE_URL": "https://api.example.com",
    "API_KEY": "prod_key_456",
    "TIMEOUT": "60"
  }
}"#
    .to_string()
}

/// 创建测试用的响应处理脚本
pub fn create_test_response_script() -> String {
    r#"
client.test("Status code is 200", function() {
    client.assert(response.status === 200, "Expected status code 200");
});

client.test("Response has correct content type", function() {
    client.assert(
        response.headers["content-type"].includes("application/json"),
        "Expected JSON content type"
    );
});

client.test("Response body is valid", function() {
    client.assert(response.body !== null, "Response body should not be null");
    client.assert(typeof response.body === "object", "Response body should be an object");
});

// 设置全局变量
client.global.set("user_id", response.body.id);
client.global.set("session_token", response.body.token);
"#
    .to_string()
}

/// 模拟HTTP响应的JSON数据
pub fn create_mock_response_json() -> serde_json::Value {
    serde_json::json!({
        "id": 123,
        "name": "test_user",
        "email": "test@example.com",
        "token": "mock_token_xyz",
        "data": {
            "items": [1, 2, 3],
            "count": 3
        }
    })
}
