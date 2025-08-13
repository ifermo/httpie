//! client模块的单元测试

use httpie::{HttpClient, HttpRequest, ResponseFormatter};
use mockito::{Matcher, Server};
use reqwest::Method;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_creation() {
        let _client = HttpClient::new();
        // 由于HttpClient的字段是私有的，我们只能测试创建是否成功
        // 这里主要测试Default trait的实现
    }

    #[test]
    fn test_http_client_default() {
        let _client1 = HttpClient::default();
        let _client2 = HttpClient::new();
        // 两种创建方式应该等效
    }

    #[test]
    fn test_http_client_with_print_response() {
        let _client = HttpClient::new().with_print_response(false);
        // 测试链式调用

        let _client2 = HttpClient::new().with_print_response(true);
        // 测试默认值设置
    }

    #[tokio::test]
    async fn test_http_client_with_script_engine() {
        let result = HttpClient::new().with_script_engine();
        assert!(
            result.is_ok(),
            "Script engine initialization should succeed"
        );
    }

    #[tokio::test]
    async fn test_execute_simple_get_request() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"message": "success"}"#)
            .create_async()
            .await;

        let request = HttpRequest::new(
            "test_get".to_string(),
            Method::GET,
            format!("{}/test", server.url()),
        );

        let mut client = HttpClient::new().with_print_response(false); // 关闭打印避免测试输出干扰
        let result = client.execute(&request).await;

        assert!(result.is_ok(), "GET request should succeed");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_post_request_with_body() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("POST", "/users")
            .match_header("content-type", "application/json")
            .match_body(Matcher::JsonString(
                r#"{"name":"test","email":"test@example.com"}"#.to_string(),
            ))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": 123, "name": "test", "email": "test@example.com"}"#)
            .create_async()
            .await;

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let request = HttpRequest::new(
            "create_user".to_string(),
            Method::POST,
            format!("{}/users", server.url()),
        )
        .with_headers(headers)
        .with_body(Some(
            r#"{"name":"test","email":"test@example.com"}"#.to_string(),
        ));

        let mut client = HttpClient::new().with_print_response(false);
        let result = client.execute(&request).await;

        assert!(result.is_ok(), "POST request should succeed");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_request_with_headers() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/protected")
            .match_header("authorization", "Bearer token123")
            .match_header("x-api-key", "api_key_456")
            .with_status(200)
            .with_body("Protected resource")
            .create_async()
            .await;

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        headers.insert("X-API-Key".to_string(), "api_key_456".to_string());

        let request = HttpRequest::new(
            "protected_request".to_string(),
            Method::GET,
            format!("{}/protected", server.url()),
        )
        .with_headers(headers);

        let mut client = HttpClient::new().with_print_response(false);
        let result = client.execute(&request).await;

        assert!(result.is_ok(), "Request with headers should succeed");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_request_with_script_engine() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/api/data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"status": "ok", "data": [1, 2, 3]}"#)
            .create_async()
            .await;

        let script = r#"
client.test("Status should be 200", function() {
    client.assert(response.status === 200, "Expected status 200");
});

client.test("Response should be JSON", function() {
    client.assert(response.contentType.includes("application/json"), "Expected JSON response");
});

client.test("Data should be array", function() {
    client.assert(Array.isArray(response.body.data), "Data should be an array");
});
"#;

        let request = HttpRequest::new(
            "test_with_script".to_string(),
            Method::GET,
            format!("{}/api/data", server.url()),
        )
        .with_response_handler(Some(script.to_string()));

        let mut client = HttpClient::new()
            .with_script_engine()
            .unwrap()
            .with_print_response(false);

        let result = client.execute(&request).await;

        assert!(result.is_ok(), "Request with script should succeed");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_execute_request_script_engine_not_initialized() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_body("test")
            .create_async()
            .await;

        let request = HttpRequest::new(
            "test_script_error".to_string(),
            Method::GET,
            format!("{}/test", server.url()),
        )
        .with_response_handler(Some("client.test('test', function() {});".to_string()));

        let mut client = HttpClient::new().with_print_response(false);
        // 注意：没有调用with_script_engine()

        let result = client.execute(&request).await;

        assert!(
            result.is_err(),
            "Should fail when script engine not initialized"
        );
        if let Err(e) = result {
            assert!(e.to_string().contains("Script engine not initialized"));
        }
    }

    #[tokio::test]
    async fn test_execute_request_network_error() {
        let request = HttpRequest::new(
            "network_error_test".to_string(),
            Method::GET,
            "http://invalid-domain-that-does-not-exist.com".to_string(),
        );

        let mut client = HttpClient::new().with_print_response(false);
        let result = client.execute(&request).await;

        assert!(result.is_err(), "Should fail for invalid domain");
    }

    #[test]
    fn test_response_formatter_creation() {
        let _formatter = ResponseFormatter::new();
        // 测试基本创建
    }

    #[test]
    fn test_response_formatter_default() {
        let _formatter1 = ResponseFormatter;
        let _formatter2 = ResponseFormatter::new();
        // 测试Default trait实现
    }

    #[tokio::test]
    async fn test_response_formatter_format_response() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/format_test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("x-custom-header", "custom-value")
            .with_body(r#"{"message": "formatted response", "code": 200}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/format_test", server.url()))
            .send()
            .await
            .unwrap();

        let formatter = ResponseFormatter::new();
        let result = formatter.format_response("format_test", response).await;

        assert!(result.is_ok(), "Response formatting should succeed");
        mock.assert_async().await;
    }

    #[test]
    fn test_response_formatter_format_test_results() {
        use httpie::TestResult;

        let formatter = ResponseFormatter::new();

        let test_results = vec![
            TestResult {
                name: "Test 1".to_string(),
                passed: true,
                message: None,
            },
            TestResult {
                name: "Test 2".to_string(),
                passed: false,
                message: Some("Assertion failed".to_string()),
            },
            TestResult {
                name: "Test 3".to_string(),
                passed: true,
                message: Some("Custom message".to_string()),
            },
        ];

        // 这个方法主要是打印输出，我们只能测试它不会panic
        formatter.format_test_results("test_request", &test_results);
    }

    #[test]
    fn test_response_formatter_format_empty_test_results() {
        let formatter = ResponseFormatter::new();
        let empty_results = vec![];

        // 空的测试结果应该不会有任何输出
        formatter.format_test_results("empty_test", &empty_results);
    }

    #[tokio::test]
    async fn test_client_builder_pattern() {
        // 测试链式调用的构建模式
        let _result = HttpClient::new()
            .with_script_engine()
            .unwrap()
            .with_print_response(false);

        // 应该能够成功创建配置好的客户端
    }

    #[tokio::test]
    async fn test_execute_all_http_methods() {
        let mut server = Server::new_async().await;

        let methods_and_paths = vec![
            (Method::GET, "/get"),
            (Method::POST, "/post"),
            (Method::PUT, "/put"),
            (Method::DELETE, "/delete"),
            (Method::PATCH, "/patch"),
            (Method::HEAD, "/head"),
            (Method::OPTIONS, "/options"),
        ];

        let mut mocks = vec![];
        for (method, path) in &methods_and_paths {
            let mock = server
                .mock(method.as_str(), *path)
                .with_status(200)
                .with_body("success")
                .create_async()
                .await;
            mocks.push(mock);
        }

        let mut client = HttpClient::new().with_print_response(false);

        for (method, path) in methods_and_paths {
            let request = HttpRequest::new(
                format!("{}_test", method.as_str().to_lowercase()),
                method.clone(),
                format!("{}{}", server.url(), path),
            );

            let result = client.execute(&request).await;
            assert!(
                result.is_ok(),
                "Request with method {} should succeed",
                method
            );
        }

        for mock in mocks {
            mock.assert_async().await;
        }
    }

    #[tokio::test]
    async fn test_execute_request_with_different_content_types() {
        let mut server = Server::new_async().await;

        // JSON response
        let json_mock = server
            .mock("GET", "/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"type": "json"}"#)
            .create_async()
            .await;

        // Plain text response
        let text_mock = server
            .mock("GET", "/text")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("plain text response")
            .create_async()
            .await;

        // XML response
        let xml_mock = server
            .mock("GET", "/xml")
            .with_status(200)
            .with_header("content-type", "application/xml")
            .with_body("<?xml version=\"1.0\"?><root><message>xml response</message></root>")
            .create_async()
            .await;

        let mut client = HttpClient::new().with_print_response(false);

        // Test JSON
        let json_request = HttpRequest::new(
            "json_test".to_string(),
            Method::GET,
            format!("{}/json", server.url()),
        );
        assert!(client.execute(&json_request).await.is_ok());

        // Test plain text
        let text_request = HttpRequest::new(
            "text_test".to_string(),
            Method::GET,
            format!("{}/text", server.url()),
        );
        assert!(client.execute(&text_request).await.is_ok());

        // Test XML
        let xml_request = HttpRequest::new(
            "xml_test".to_string(),
            Method::GET,
            format!("{}/xml", server.url()),
        );
        assert!(client.execute(&xml_request).await.is_ok());

        json_mock.assert_async().await;
        text_mock.assert_async().await;
        xml_mock.assert_async().await;
    }
}
