//! script模块的单元测试

use httpie::{ResponseObject, ScriptEngine, TestResult};
use mockito::Server;

use serde_json::{Value, json};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_engine_creation() {
        let result = ScriptEngine::new();
        assert!(result.is_ok(), "ScriptEngine creation should succeed");
    }

    #[test]
    fn test_script_engine_default() {
        let _engine = ScriptEngine::default();
        // 测试Default trait实现
    }

    #[tokio::test]
    async fn test_execute_simple_test_script() {
        let mut engine = ScriptEngine::new().unwrap();

        let response_obj = create_test_response_object();

        let script = r#"
client.test("Simple test", function() {
    client.assert(true, "This should always pass");
});
"#;

        let result = engine
            .execute_response_script(script.to_string(), response_obj)
            .await;
        assert!(result.is_ok(), "Simple script execution should succeed");

        let test_results = result.unwrap();
        assert_eq!(test_results.len(), 1);
        assert_eq!(test_results[0].name, "Simple test");
        assert!(test_results[0].passed);
        assert!(test_results[0].message.is_none());
    }

    #[tokio::test]
    async fn test_execute_failing_test_script() {
        let mut engine = ScriptEngine::new().unwrap();

        let response_obj = create_test_response_object();

        let script = r#"
client.test("Failing test", function() {
    client.assert(false, "This should always fail");
});
"#;

        let result = engine
            .execute_response_script(script.to_string(), response_obj)
            .await;
        assert!(
            result.is_ok(),
            "Script execution should succeed even with failing tests"
        );

        let test_results = result.unwrap();
        assert_eq!(test_results.len(), 1);
        assert_eq!(test_results[0].name, "Failing test");
        assert!(!test_results[0].passed);
        assert!(test_results[0].message.is_some());
        assert!(
            test_results[0]
                .message
                .as_ref()
                .unwrap()
                .contains("This should always fail")
        );
    }

    #[tokio::test]
    async fn test_execute_multiple_tests() {
        let mut engine = ScriptEngine::new().unwrap();

        let response_obj = create_test_response_object();

        let script = r#"
client.test("Test 1", function() {
    client.assert(response.status === 200, "Status should be 200");
});

client.test("Test 2", function() {
    client.assert(response.body.message === "success", "Message should be success");
});

client.test("Test 3", function() {
    client.assert(response.headers["content-type"] === "application/json", "Content type should be JSON");
});
"#;

        let result = engine
            .execute_response_script(script.to_string(), response_obj)
            .await;
        assert!(result.is_ok(), "Multiple tests script should succeed");

        let test_results = result.unwrap();
        assert_eq!(test_results.len(), 3);

        for test_result in &test_results {
            assert!(test_result.passed, "All tests should pass");
        }
    }

    #[tokio::test]
    async fn test_execute_script_with_global_variables() {
        let mut engine = ScriptEngine::new().unwrap();

        let response_obj = create_test_response_object();

        let script = r#"
client.global.set("userId", response.body.id);
client.global.set("token", response.body.token);
client.global.set("timestamp", Date.now());

client.test("Global variables test", function() {
    client.assert(client.global.get("userId") === 123, "User ID should be set");
    client.assert(client.global.get("token") === "abc123", "Token should be set");
});
"#;

        let result = engine
            .execute_response_script(script.to_string(), response_obj)
            .await;
        assert!(
            result.is_ok(),
            "Script with global variables should succeed"
        );

        let test_results = result.unwrap();
        assert_eq!(test_results.len(), 1);
        assert!(test_results[0].passed);

        // 检查全局变量是否被正确设置
        assert_eq!(engine.get_global_variable("userId"), Some(&json!(123)));
        assert_eq!(engine.get_global_variable("token"), Some(&json!("abc123")));
        assert!(engine.get_global_variable("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_execute_script_with_response_validation() {
        let mut engine = ScriptEngine::new().unwrap();

        let response_obj = create_test_response_object();

        let script = r#"
client.test("Response status validation", function() {
    client.assert(response.status === 200, "Status should be 200");
});

client.test("Response headers validation", function() {
    client.assert(response.headers["content-type"] === "application/json", "Content-Type should be JSON");
    client.assert(response.headers["x-custom-header"] === "custom-value", "Custom header should be present");
});

client.test("Response body validation", function() {
    client.assert(typeof response.body === "object", "Body should be an object");
    client.assert(response.body.message === "success", "Message should be success");
    client.assert(response.body.id === 123, "ID should be 123");
    client.assert(response.body.token === "abc123", "Token should be abc123");
});

client.test("Content type validation", function() {
    client.assert(response.contentType === "application/json", "Content type should match");
});
"#;

        let result = engine
            .execute_response_script(script.to_string(), response_obj)
            .await;
        assert!(result.is_ok(), "Response validation script should succeed");

        let test_results = result.unwrap();
        assert_eq!(test_results.len(), 4);

        for test_result in &test_results {
            assert!(
                test_result.passed,
                "Test '{}' should pass",
                test_result.name
            );
        }
    }

    #[tokio::test]
    async fn test_execute_script_with_syntax_error() {
        let mut engine = ScriptEngine::new().unwrap();

        let response_obj = create_test_response_object();

        let script = r#"
client.test("Syntax error test", function() {
    // 故意的语法错误
    client.assert(response.status === 200
});
"#;

        let result = engine
            .execute_response_script(script.to_string(), response_obj)
            .await;
        assert!(result.is_err(), "Script with syntax error should fail");

        if let Err(e) = result {
            assert!(e.to_string().contains("Script execution failed"));
        }
    }

    #[tokio::test]
    async fn test_execute_script_with_runtime_error() {
        let mut engine = ScriptEngine::new().unwrap();

        let response_obj = create_test_response_object();

        let script = r#"
client.test("Runtime error test", function() {
    // 访问不存在的属性会导致运行时错误
    client.assert(response.nonExistentProperty.value === "test");
});
"#;

        let result = engine
            .execute_response_script(script.to_string(), response_obj)
            .await;
        assert!(result.is_ok(), "Script execution should succeed");

        let test_results = result.unwrap();
        assert_eq!(test_results.len(), 1);
        assert!(
            !test_results[0].passed,
            "Test should fail due to runtime error"
        );
        assert!(test_results[0].message.is_some());
    }

    #[tokio::test]
    async fn test_execute_empty_script() {
        let mut engine = ScriptEngine::new().unwrap();

        let response_obj = create_test_response_object();

        let script = "";

        let result = engine
            .execute_response_script(script.to_string(), response_obj)
            .await;
        assert!(result.is_ok(), "Empty script should succeed");

        let test_results = result.unwrap();
        assert!(
            test_results.is_empty(),
            "Empty script should produce no test results"
        );
    }

    #[tokio::test]
    async fn test_execute_script_with_console_log() {
        let mut engine = ScriptEngine::new().unwrap();

        let response_obj = create_test_response_object();

        let script = r#"
console.log("This is a log message");
console.log("Response status:", response.status);
console.log("Response body:", response.body);

client.test("Console log test", function() {
    client.assert(true, "This should pass");
});
"#;

        let result = engine
            .execute_response_script(script.to_string(), response_obj)
            .await;
        assert!(result.is_ok(), "Script with console.log should succeed");

        let test_results = result.unwrap();
        assert_eq!(test_results.len(), 1);
        assert!(test_results[0].passed);
    }

    #[tokio::test]
    async fn test_get_all_global_variables() {
        let mut engine = ScriptEngine::new().unwrap();

        let response_obj = create_test_response_object();

        let script = r#"
client.global.set("var1", "value1");
client.global.set("var2", 42);
client.global.set("var3", {"nested": "object"});
"#;

        let result = engine
            .execute_response_script(script.to_string(), response_obj)
            .await;
        assert!(result.is_ok());

        let all_vars = engine.get_all_global_variables();
        assert_eq!(all_vars.len(), 3);
        assert_eq!(all_vars.get("var1"), Some(&json!("value1")));
        assert_eq!(all_vars.get("var2"), Some(&json!(42)));
        assert_eq!(all_vars.get("var3"), Some(&json!({"nested": "object"})));
    }

    #[tokio::test]
    async fn test_response_object_from_response() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/test")
            .with_status(201)
            .with_header("content-type", "application/json; charset=utf-8")
            .with_header("x-custom-header", "test-value")
            .with_body(r#"{"result": "created", "id": 456}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/test", server.url()))
            .send()
            .await
            .unwrap();

        let response_obj = ResponseObject::from_response(response).await.unwrap();

        assert_eq!(response_obj.status, 201);
        assert_eq!(response_obj.content_type, "application/json; charset=utf-8");
        assert!(response_obj.headers.contains_key("content-type"));
        assert!(response_obj.headers.contains_key("x-custom-header"));
        assert_eq!(
            response_obj.headers.get("x-custom-header").unwrap(),
            "test-value"
        );

        // 验证body被正确解析为JSON
        if let Value::Object(body_obj) = &response_obj.body {
            assert_eq!(body_obj.get("result").unwrap(), &json!("created"));
            assert_eq!(body_obj.get("id").unwrap(), &json!(456));
        } else {
            panic!("Body should be parsed as JSON object");
        }

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_response_object_with_non_json_body() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/text")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("This is plain text")
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/text", server.url()))
            .send()
            .await
            .unwrap();

        let response_obj = ResponseObject::from_response(response).await.unwrap();

        assert_eq!(response_obj.status, 200);
        assert_eq!(response_obj.content_type, "text/plain");

        // 非JSON内容应该被存储为字符串
        if let Value::String(body_str) = &response_obj.body {
            assert_eq!(body_str, "This is plain text");
        } else {
            panic!("Non-JSON body should be stored as string");
        }

        mock.assert_async().await;
    }

    #[test]
    fn test_test_result_creation() {
        let test_result = TestResult {
            name: "Test Name".to_string(),
            passed: true,
            message: Some("Test message".to_string()),
        };

        assert_eq!(test_result.name, "Test Name");
        assert!(test_result.passed);
        assert_eq!(test_result.message, Some("Test message".to_string()));
    }

    #[test]
    fn test_test_result_serialization() {
        let test_result = TestResult {
            name: "Serialization Test".to_string(),
            passed: false,
            message: Some("Error message".to_string()),
        };

        let json_str = serde_json::to_string(&test_result).unwrap();
        let deserialized: TestResult = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.name, test_result.name);
        assert_eq!(deserialized.passed, test_result.passed);
        assert_eq!(deserialized.message, test_result.message);
    }

    #[test]
    fn test_response_object_clone() {
        let response_obj = create_test_response_object();
        let cloned = response_obj.clone();

        assert_eq!(response_obj.status, cloned.status);
        assert_eq!(response_obj.content_type, cloned.content_type);
        assert_eq!(response_obj.headers, cloned.headers);
        assert_eq!(response_obj.body, cloned.body);
    }

    #[test]
    fn test_response_object_debug() {
        let response_obj = create_test_response_object();
        let debug_str = format!("{:?}", response_obj);

        assert!(debug_str.contains("ResponseObject"));
        assert!(debug_str.contains("200"));
        assert!(debug_str.contains("application/json"));
    }

    // 辅助函数：创建测试用的ResponseObject
    fn create_test_response_object() -> ResponseObject {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("x-custom-header".to_string(), "custom-value".to_string());

        ResponseObject {
            status: 200,
            headers,
            body: json!({
                "message": "success",
                "id": 123,
                "token": "abc123",
                "data": {
                    "items": [1, 2, 3],
                    "count": 3
                }
            }),
            content_type: "application/json".to_string(),
        }
    }
}
