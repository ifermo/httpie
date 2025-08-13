//! parser模块的单元测试

use httpie::{Environment, HttpParser};
use reqwest::Method;
use std::fs;
use tempfile::NamedTempFile;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let env = Environment::new();
        let _parser = HttpParser::new(env);

        // 基本创建测试
        // 由于HttpParser的字段是私有的，我们只能通过其行为来测试
    }

    #[test]
    fn test_parse_simple_get_request() {
        let content = r#"
### Simple GET Request
GET https://httpbin.org/get
User-Agent: httpie-test
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let requests = parser
            .parse_file(&temp_file.path().to_string_lossy())
            .unwrap();

        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.name, "Simple GET Request");
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.url, "https://httpbin.org/get");
        assert_eq!(request.headers.len(), 1);
        assert_eq!(request.headers.get("User-Agent").unwrap(), "httpie-test");
        assert!(request.body.is_none());
        assert!(request.response_handler.is_none());
    }

    #[test]
    fn test_parse_post_request_with_body() {
        let content = r#"
### POST Request with JSON Body
POST https://httpbin.org/post
Content-Type: application/json
Authorization: Bearer token123

{
  "name": "test",
  "value": 42
}
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let requests = parser
            .parse_file(&temp_file.path().to_string_lossy())
            .unwrap();

        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.name, "POST Request with JSON Body");
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.url, "https://httpbin.org/post");
        assert_eq!(request.headers.len(), 2);
        assert_eq!(
            request.headers.get("Content-Type").unwrap(),
            "application/json"
        );
        assert_eq!(
            request.headers.get("Authorization").unwrap(),
            "Bearer token123"
        );

        let expected_body = "{\n  \"name\": \"test\",\n  \"value\": 42\n}";
        assert_eq!(request.body.as_ref().unwrap(), expected_body);
        assert!(request.response_handler.is_none());
    }

    #[test]
    fn test_parse_request_with_variables() {
        let content = r#"
@baseUrl = https://api.example.com
@token = abc123

### Request with Variables
GET {{baseUrl}}/users
Authorization: Bearer {{token}}
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let requests = parser
            .parse_file(&temp_file.path().to_string_lossy())
            .unwrap();

        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.name, "Request with Variables");
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.url, "https://api.example.com/users");
        assert_eq!(
            request.headers.get("Authorization").unwrap(),
            "Bearer abc123"
        );
    }

    #[test]
    fn test_parse_request_with_response_handler() {
        let content = r#"
### Request with Response Handler
GET https://httpbin.org/get

> {%
client.test("Status should be 200", function() {
    client.assert(response.status === 200);
});
%}
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let requests = parser
            .parse_file(&temp_file.path().to_string_lossy())
            .unwrap();

        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.name, "Request with Response Handler");
        assert!(
            request.response_handler.is_some(),
            "Response handler should be parsed"
        );

        let script = request.response_handler.as_ref().unwrap();
        assert!(script.contains("client.test"));
        assert!(script.contains("response.status === 200"));
    }

    #[test]
    fn test_parse_multiple_requests() {
        let content = r#"
@baseUrl = https://httpbin.org
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

> {%
client.test("Status should be 200", function() {
    client.assert(response.status === 200, "Expected status 200");
});
%}

### Test PUT Request
PUT {{baseUrl}}/put
Content-Type: application/json

{
  "id": 1,
  "updated": true
}
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let requests = parser
            .parse_file(&temp_file.path().to_string_lossy())
            .unwrap();

        assert!(requests.len() >= 2); // 至少有GET和POST请求

        // 验证第一个请求
        let get_request = requests.iter().find(|r| r.name.contains("GET")).unwrap();
        assert_eq!(get_request.method, Method::GET);
        assert!(get_request.url.contains("httpbin.org"));

        // 验证POST请求
        let post_request = requests.iter().find(|r| r.name.contains("POST")).unwrap();
        assert_eq!(post_request.method, Method::POST);
        assert!(post_request.body.is_some());
        assert!(
            post_request.response_handler.is_some(),
            "POST request should have response handler"
        );
    }

    #[test]
    fn test_parse_file_not_found() {
        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let result = parser.parse_file("/nonexistent/file.http");
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("File not found"));
        }
    }

    #[test]
    fn test_parse_empty_file() {
        let content = "";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let requests = parser
            .parse_file(&temp_file.path().to_string_lossy())
            .unwrap();
        assert!(requests.is_empty());
    }

    #[test]
    fn test_parse_file_with_only_variables() {
        let content = r#"
@baseUrl = https://api.example.com
@token = abc123
@version = v1
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let requests = parser
            .parse_file(&temp_file.path().to_string_lossy())
            .unwrap();
        assert!(requests.is_empty());
    }

    #[test]
    fn test_parse_file_with_comments_only() {
        let content = r#"
# This is a comment
// This is also a comment
### This looks like a request but has no HTTP method
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let requests = parser
            .parse_file(&temp_file.path().to_string_lossy())
            .unwrap();
        assert!(requests.is_empty());
    }

    #[test]
    fn test_parse_invalid_http_method() {
        let content = r#"
### Invalid Method Request
INVALID_METHOD https://httpbin.org/get
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let result = parser.parse_file(&temp_file.path().to_string_lossy());
        // 根据parser实现，无效方法会导致请求被跳过，返回空的请求列表
        match result {
            Ok(requests) => {
                // 如果解析成功，应该没有请求被解析出来
                assert!(
                    requests.is_empty(),
                    "Invalid method should result in no parsed requests"
                );
            }
            Err(e) => {
                // 如果返回错误，应该是InvalidMethod错误
                assert!(e.to_string().contains("Invalid") || e.to_string().contains("method"));
            }
        }
    }

    #[test]
    fn test_parse_malformed_request_line() {
        let content = r#"
### Malformed Request
GET
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let result = parser.parse_file(&temp_file.path().to_string_lossy());
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("Invalid request"));
        }
    }

    #[test]
    fn test_parse_request_with_environment_variables() {
        let content = r#"
### Request with Env Variables
GET {{BASE_URL}}/api/{{VERSION}}/users
Authorization: Bearer {{API_KEY}}
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let mut env = Environment::new();
        env.insert("BASE_URL".to_string(), "https://prod.api.com".to_string());
        env.insert("VERSION".to_string(), "v2".to_string());
        env.insert("API_KEY".to_string(), "prod_key_456".to_string());

        let mut parser = HttpParser::new(env);

        let requests = parser
            .parse_file(&temp_file.path().to_string_lossy())
            .unwrap();

        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.url, "https://prod.api.com/api/v2/users");
        assert_eq!(
            request.headers.get("Authorization").unwrap(),
            "Bearer prod_key_456"
        );
    }

    #[test]
    fn test_parse_request_with_mixed_variables() {
        let content = r#"
@fileVar = file_value

### Mixed Variables Request
GET {{BASE_URL}}/{{fileVar}}
X-File-Var: {{fileVar}}
X-Env-Var: {{ENV_VAR}}
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let mut env = Environment::new();
        env.insert("BASE_URL".to_string(), "https://api.com".to_string());
        env.insert("ENV_VAR".to_string(), "env_value".to_string());

        let mut parser = HttpParser::new(env);

        let requests = parser
            .parse_file(&temp_file.path().to_string_lossy())
            .unwrap();

        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.url, "https://api.com/file_value");
        assert_eq!(request.headers.get("X-File-Var").unwrap(), "file_value");
        assert_eq!(request.headers.get("X-Env-Var").unwrap(), "env_value");
    }

    #[test]
    fn test_parse_all_http_methods() {
        let content = r#"
### GET Request
GET https://httpbin.org/get

### POST Request
POST https://httpbin.org/post

### PUT Request
PUT https://httpbin.org/put

### DELETE Request
DELETE https://httpbin.org/delete

### PATCH Request
PATCH https://httpbin.org/patch

### HEAD Request
HEAD https://httpbin.org/get

### OPTIONS Request
OPTIONS https://httpbin.org/get
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let requests = parser
            .parse_file(&temp_file.path().to_string_lossy())
            .unwrap();

        assert_eq!(requests.len(), 7);

        let methods = [
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::HEAD,
            Method::OPTIONS,
        ];

        for (i, expected_method) in methods.iter().enumerate() {
            assert_eq!(requests[i].method, *expected_method);
        }
    }

    #[test]
    fn test_parse_request_with_complex_body_and_script() {
        let content = r#"
### Complex Request
POST https://api.example.com/users
Content-Type: application/json
Authorization: Bearer token123

{
  "user": {
    "name": "John Doe",
    "email": "john@example.com",
    "preferences": {
      "theme": "dark",
      "notifications": true
    }
  }
}

> {%
client.test("User creation successful", function() {
    client.assert(response.status === 201, "Expected 201 Created");
    client.assert(response.body.user.id !== undefined, "User ID should be present");
});

client.test("Response has correct structure", function() {
    client.assert(typeof response.body.user === "object", "User should be an object");
    client.assert(response.body.user.name === "John Doe", "Name should match");
});

// Set global variables for subsequent requests
client.global.set("userId", response.body.user.id);
client.global.set("userToken", response.body.token);
%}
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), content).unwrap();

        let env = Environment::new();
        let mut parser = HttpParser::new(env);

        let requests = parser
            .parse_file(&temp_file.path().to_string_lossy())
            .unwrap();

        assert_eq!(requests.len(), 1);
        let request = &requests[0];

        assert_eq!(request.name, "Complex Request");
        assert_eq!(request.method, Method::POST);
        assert!(request.body.is_some());
        assert!(
            request.response_handler.is_some(),
            "Complex request should have response handler"
        );

        let body = request.body.as_ref().unwrap();
        assert!(body.contains("John Doe"));
        assert!(body.contains("preferences"));

        let script = request.response_handler.as_ref().unwrap();
        assert!(script.contains("User creation successful"));
        assert!(script.contains("client.global.set"));
    }
}
