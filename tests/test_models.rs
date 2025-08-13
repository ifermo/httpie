//! models模块的单元测试

use httpie::{Environment, HttpRequest};
use reqwest::Method;
use std::collections::HashMap;
use std::fs;

use tempfile::NamedTempFile;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_creation() {
        let request = HttpRequest::new(
            "test_request".to_string(),
            Method::GET,
            "https://example.com".to_string(),
        );

        assert_eq!(request.name, "test_request");
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.url, "https://example.com");
        assert!(request.headers.is_empty());
        assert!(request.body.is_none());
        assert!(request.response_handler.is_none());
    }

    #[test]
    fn test_http_request_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Authorization".to_string(), "Bearer token".to_string());

        let request = HttpRequest::new(
            "test_request".to_string(),
            Method::POST,
            "https://example.com/api".to_string(),
        )
        .with_headers(headers.clone());

        assert_eq!(request.headers, headers);
        assert_eq!(request.headers.len(), 2);
        assert_eq!(
            request.headers.get("Content-Type").unwrap(),
            "application/json"
        );
        assert_eq!(
            request.headers.get("Authorization").unwrap(),
            "Bearer token"
        );
    }

    #[test]
    fn test_http_request_with_body() {
        let body = "{\"key\": \"value\"}".to_string();
        let request = HttpRequest::new(
            "test_request".to_string(),
            Method::POST,
            "https://example.com/api".to_string(),
        )
        .with_body(Some(body.clone()));

        assert_eq!(request.body, Some(body));
    }

    #[test]
    fn test_http_request_with_response_handler() {
        let script = "client.test('test', function() { client.assert(true); });".to_string();
        let request = HttpRequest::new(
            "test_request".to_string(),
            Method::GET,
            "https://example.com".to_string(),
        )
        .with_response_handler(Some(script.clone()));

        assert_eq!(request.response_handler, Some(script));
    }

    #[test]
    fn test_http_request_builder_pattern() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let body = "{\"test\": true}".to_string();
        let script =
            "client.test('status', function() { client.assert(response.status === 200); });"
                .to_string();

        let request = HttpRequest::new(
            "complex_request".to_string(),
            Method::PUT,
            "https://api.example.com/users/1".to_string(),
        )
        .with_headers(headers.clone())
        .with_body(Some(body.clone()))
        .with_response_handler(Some(script.clone()));

        assert_eq!(request.name, "complex_request");
        assert_eq!(request.method, Method::PUT);
        assert_eq!(request.url, "https://api.example.com/users/1");
        assert_eq!(request.headers, headers);
        assert_eq!(request.body, Some(body));
        assert_eq!(request.response_handler, Some(script));
    }

    #[test]
    fn test_environment_creation() {
        let env = Environment::new();
        assert!(env.variables().is_empty());
    }

    #[test]
    fn test_environment_insert_and_get() {
        let mut env = Environment::new();
        env.insert("API_KEY".to_string(), "test_key_123".to_string());
        env.insert(
            "BASE_URL".to_string(),
            "https://api.example.com".to_string(),
        );

        assert_eq!(env.get("API_KEY"), Some(&"test_key_123".to_string()));
        assert_eq!(
            env.get("BASE_URL"),
            Some(&"https://api.example.com".to_string())
        );
        assert_eq!(env.get("NON_EXISTENT"), None);
        assert_eq!(env.variables().len(), 2);
    }

    #[test]
    fn test_environment_extend() {
        let mut env = Environment::new();
        env.insert("EXISTING_KEY".to_string(), "existing_value".to_string());

        let mut new_vars = HashMap::new();
        new_vars.insert("NEW_KEY1".to_string(), "new_value1".to_string());
        new_vars.insert("NEW_KEY2".to_string(), "new_value2".to_string());
        new_vars.insert("EXISTING_KEY".to_string(), "updated_value".to_string());

        env.extend(new_vars);

        assert_eq!(env.variables().len(), 3);
        assert_eq!(env.get("EXISTING_KEY"), Some(&"updated_value".to_string()));
        assert_eq!(env.get("NEW_KEY1"), Some(&"new_value1".to_string()));
        assert_eq!(env.get("NEW_KEY2"), Some(&"new_value2".to_string()));
    }

    #[test]
    fn test_environment_from_file_success() {
        let env_content = r#"{
  "development": {
    "API_KEY": "dev_key_123",
    "BASE_URL": "https://dev.api.example.com",
    "DEBUG": "true"
  },
  "production": {
    "API_KEY": "prod_key_456",
    "BASE_URL": "https://api.example.com",
    "DEBUG": "false"
  }
}"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), env_content).unwrap();

        let env = Environment::from_file(&temp_file.path().to_string_lossy()).unwrap();

        // 默认使用development环境
        assert_eq!(env.get("API_KEY"), Some(&"dev_key_123".to_string()));
        assert_eq!(
            env.get("BASE_URL"),
            Some(&"https://dev.api.example.com".to_string())
        );
        assert_eq!(env.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(env.variables().len(), 3);
    }

    #[test]
    fn test_environment_from_file_not_found() {
        let result = Environment::from_file("/non/existent/file.json");
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("File not found"));
        }
    }

    #[test]
    fn test_environment_from_file_invalid_json() {
        let invalid_json = "{ invalid json content }";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), invalid_json).unwrap();

        let result = Environment::from_file(&temp_file.path().to_string_lossy());
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("JSON parsing error"));
        }
    }

    #[test]
    fn test_environment_from_file_missing_development_env() {
        let env_content = r#"{
  "production": {
    "API_KEY": "prod_key_456",
    "BASE_URL": "https://api.example.com"
  }
}"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), env_content).unwrap();

        let env = Environment::from_file(&temp_file.path().to_string_lossy()).unwrap();

        // 如果没有development环境，应该返回空的环境
        assert!(env.variables().is_empty());
    }

    #[test]
    fn test_environment_variables_reference() {
        let mut env = Environment::new();
        env.insert("KEY1".to_string(), "value1".to_string());
        env.insert("KEY2".to_string(), "value2".to_string());

        let vars = env.variables();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains_key("KEY1"));
        assert!(vars.contains_key("KEY2"));
        assert_eq!(vars.get("KEY1").unwrap(), "value1");
        assert_eq!(vars.get("KEY2").unwrap(), "value2");
    }

    #[test]
    fn test_environment_default_trait() {
        let env1 = Environment::default();
        let env2 = Environment::new();

        assert!(env1.variables().is_empty());
        assert!(env2.variables().is_empty());
        assert_eq!(env1.variables().len(), env2.variables().len());
    }

    #[test]
    fn test_environment_clone() {
        let mut env1 = Environment::new();
        env1.insert("TEST_KEY".to_string(), "test_value".to_string());

        let env2 = env1.clone();

        assert_eq!(env1.get("TEST_KEY"), env2.get("TEST_KEY"));
        assert_eq!(env1.variables().len(), env2.variables().len());
    }
}
