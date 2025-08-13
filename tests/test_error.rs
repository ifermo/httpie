//! error模块的单元测试

use httpie::HttpieError;
use std::io;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let httpie_err = HttpieError::Io(io_err);

        assert!(httpie_err.to_string().contains("IO error"));
        assert!(httpie_err.to_string().contains("File not found"));
    }

    #[test]
    fn test_http_error() {
        // 创建一个模拟的reqwest错误
        let url = "https://invalid-url-that-does-not-exist.com";
        let client = reqwest::Client::new();

        // 这个测试需要在异步环境中运行
        tokio_test::block_on(async {
            let result = client.get(url).send().await;
            if let Err(reqwest_err) = result {
                let httpie_err = HttpieError::Http(reqwest_err);
                assert!(httpie_err.to_string().contains("HTTP error"));
            }
        });
    }

    #[test]
    fn test_json_parsing_error() {
        let invalid_json = "{ invalid json }";
        let json_err = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let httpie_err = HttpieError::Json(json_err);

        assert!(httpie_err.to_string().contains("JSON parsing error"));
    }

    #[test]
    fn test_parse_error() {
        let error_msg = "Failed to parse HTTP request";
        let httpie_err = HttpieError::Parse(error_msg.to_string());

        assert_eq!(
            httpie_err.to_string(),
            "Parse error: Failed to parse HTTP request"
        );
    }

    #[test]
    fn test_invalid_method_error() {
        let invalid_method = "INVALID_METHOD";
        let httpie_err = HttpieError::InvalidMethod(invalid_method.to_string());

        assert_eq!(
            httpie_err.to_string(),
            "Invalid HTTP method: INVALID_METHOD"
        );
    }

    #[test]
    fn test_file_not_found_error() {
        let file_path = "/path/to/nonexistent/file.http";
        let httpie_err = HttpieError::FileNotFound(file_path.to_string());

        assert_eq!(
            httpie_err.to_string(),
            "File not found: /path/to/nonexistent/file.http"
        );
    }

    #[test]
    fn test_invalid_request_error() {
        let error_msg = "Missing required field: URL";
        let httpie_err = HttpieError::InvalidRequest(error_msg.to_string());

        assert_eq!(
            httpie_err.to_string(),
            "Invalid request format: Missing required field: URL"
        );
    }

    #[test]
    fn test_script_error() {
        let error_msg = "JavaScript execution failed: ReferenceError: undefined variable";
        let httpie_err = HttpieError::ScriptError(error_msg.to_string());

        assert_eq!(
            httpie_err.to_string(),
            "Script execution error: JavaScript execution failed: ReferenceError: undefined variable"
        );
    }

    #[test]
    fn test_script_parsing_error() {
        let error_msg = "Invalid JavaScript syntax at line 5";
        let httpie_err = HttpieError::ScriptParsingError(error_msg.to_string());

        assert_eq!(
            httpie_err.to_string(),
            "Script parsing error: Invalid JavaScript syntax at line 5"
        );
    }

    #[test]
    fn test_error_debug_format() {
        let httpie_err = HttpieError::Parse("test error".to_string());
        let debug_str = format!("{:?}", httpie_err);

        assert!(debug_str.contains("Parse"));
        assert!(debug_str.contains("test error"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
        let httpie_err: HttpieError = io_err.into();

        match httpie_err {
            HttpieError::Io(_) => {} // 正确的转换
            _ => panic!("Expected HttpieError::Io variant"),
        }
    }

    #[test]
    fn test_error_from_serde_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let httpie_err: HttpieError = json_err.into();

        match httpie_err {
            HttpieError::Json(_) => {} // 正确的转换
            _ => panic!("Expected HttpieError::Json variant"),
        }
    }

    #[test]
    fn test_error_chain() {
        // 测试错误链
        let io_err = io::Error::new(io::ErrorKind::NotFound, "Original IO error");
        let httpie_err = HttpieError::Io(io_err);

        // 检查错误源
        let error_string = httpie_err.to_string();
        assert!(error_string.contains("IO error"));
        assert!(error_string.contains("Original IO error"));
    }

    #[test]
    fn test_all_error_variants_display() {
        let errors = vec![
            HttpieError::Parse("parse error".to_string()),
            HttpieError::InvalidMethod("INVALID".to_string()),
            HttpieError::FileNotFound("file.http".to_string()),
            HttpieError::InvalidRequest("bad request".to_string()),
            HttpieError::ScriptError("script error".to_string()),
            HttpieError::ScriptParsingError("parsing error".to_string()),
        ];

        for error in errors {
            let error_str = error.to_string();
            assert!(!error_str.is_empty(), "Error string should not be empty");
            assert!(error_str.len() > 5, "Error string should be descriptive");
        }
    }

    #[test]
    fn test_error_send_sync() {
        // 确保错误类型实现了Send和Sync trait
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HttpieError>();
    }

    #[test]
    fn test_result_type_alias() {
        // 测试Result类型别名
        fn test_function() -> httpie::Result<String> {
            Ok("success".to_string())
        }

        let result = test_function();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_result_type_alias_error() {
        // 测试Result类型别名返回错误
        fn test_function() -> httpie::Result<String> {
            Err(HttpieError::Parse("test error".to_string()))
        }

        let result = test_function();
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("Parse error"));
        }
    }
}
