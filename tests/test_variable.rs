//! variable模块的单元测试

use httpie::{Environment, VariableReplacer};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_replacer_creation() {
        let env = Environment::new();
        let replacer = VariableReplacer::new(&env);

        // 测试基本创建
        let result = replacer.replace("no variables here");
        assert_eq!(result, "no variables here");
    }

    #[test]
    fn test_simple_variable_replacement() {
        let mut env = Environment::new();
        env.insert("baseUrl".to_string(), "https://api.example.com".to_string());
        env.insert("version".to_string(), "v1".to_string());

        let replacer = VariableReplacer::new(&env);

        let result = replacer.replace("{{baseUrl}}/{{version}}/users");
        assert_eq!(result, "https://api.example.com/v1/users");
    }

    #[test]
    fn test_single_variable_replacement() {
        let mut env = Environment::new();
        env.insert("token".to_string(), "abc123".to_string());

        let replacer = VariableReplacer::new(&env);

        let result = replacer.replace("Bearer {{token}}");
        assert_eq!(result, "Bearer abc123");
    }

    #[test]
    fn test_multiple_same_variable() {
        let mut env = Environment::new();
        env.insert("name".to_string(), "test".to_string());

        let replacer = VariableReplacer::new(&env);

        let result = replacer.replace("{{name}}_{{name}}_{{name}}");
        assert_eq!(result, "test_test_test");
    }

    #[test]
    fn test_undefined_variable() {
        let env = Environment::new();
        let replacer = VariableReplacer::new(&env);

        let result = replacer.replace("{{undefined_variable}}");
        assert_eq!(result, "{{undefined_variable}}"); // 保持原样
    }

    #[test]
    fn test_mixed_defined_undefined_variables() {
        let mut env = Environment::new();
        env.insert("defined".to_string(), "value".to_string());

        let replacer = VariableReplacer::new(&env);

        let result = replacer.replace("{{defined}} and {{undefined}}");
        assert_eq!(result, "value and {{undefined}}");
    }

    #[test]
    fn test_empty_variable_name() {
        let env = Environment::new();
        let replacer = VariableReplacer::new(&env);

        let result = replacer.replace("{{}}");
        assert_eq!(result, "{{}}"); // 保持原样
    }

    #[test]
    fn test_malformed_variable_syntax() {
        let mut env = Environment::new();
        env.insert("test".to_string(), "value".to_string());

        let replacer = VariableReplacer::new(&env);

        // 测试各种格式错误的变量语法
        assert_eq!(replacer.replace("{test}"), "{test}"); // 单个大括号
        assert_eq!(replacer.replace("{{test}"), "{{test}"); // 缺少结束大括号
        assert_eq!(replacer.replace("{test}}"), "{test}}"); // 缺少开始大括号
        assert_eq!(replacer.replace("{{test"), "{{test"); // 完全缺少结束
    }

    #[test]
    fn test_nested_braces() {
        let mut env = Environment::new();
        env.insert("outer".to_string(), "{{inner}}".to_string());
        env.insert("inner".to_string(), "value".to_string());

        let replacer = VariableReplacer::new(&env);
        let result = replacer.replace("{{outer}}");

        // 当前实现支持递归替换，{{outer}}被替换为"{{inner}}"，然后{{inner}}被替换为"value"
        assert_eq!(result, "value");
    }

    #[test]
    fn test_variable_with_special_characters() {
        let mut env = Environment::new();
        env.insert(
            "special_var".to_string(),
            "value with spaces and symbols!@#$%".to_string(),
        );
        env.insert(
            "url".to_string(),
            "https://example.com/path?param=value&other=123".to_string(),
        );

        let replacer = VariableReplacer::new(&env);

        let result1 = replacer.replace("{{special_var}}");
        assert_eq!(result1, "value with spaces and symbols!@#$%");

        let result2 = replacer.replace("{{url}}");
        assert_eq!(result2, "https://example.com/path?param=value&other=123");
    }

    #[test]
    fn test_variable_in_json_body() {
        let mut env = Environment::new();
        env.insert("userId".to_string(), "12345".to_string());
        env.insert("userName".to_string(), "john_doe".to_string());

        let replacer = VariableReplacer::new(&env);

        let json_template = r#"{
  "id": "{{userId}}",
  "name": "{{userName}}",
  "active": true
}"#;

        let result = replacer.replace(json_template);
        let expected = r#"{
  "id": "12345",
  "name": "john_doe",
  "active": true
}"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn test_variable_in_headers() {
        let mut env = Environment::new();
        env.insert("authToken".to_string(), "Bearer abc123xyz".to_string());
        env.insert("contentType".to_string(), "application/json".to_string());

        let replacer = VariableReplacer::new(&env);

        let auth_header = replacer.replace("Authorization: {{authToken}}");
        assert_eq!(auth_header, "Authorization: Bearer abc123xyz");

        let content_header = replacer.replace("Content-Type: {{contentType}}");
        assert_eq!(content_header, "Content-Type: application/json");
    }

    #[test]
    fn test_empty_string_replacement() {
        let env = Environment::new();
        let replacer = VariableReplacer::new(&env);

        let result = replacer.replace("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_no_variables_in_string() {
        let env = Environment::new();
        let replacer = VariableReplacer::new(&env);

        let input = "This is a plain string with no variables";
        let result = replacer.replace(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_variable_with_numbers() {
        let mut env = Environment::new();
        env.insert("var1".to_string(), "first".to_string());
        env.insert("var2".to_string(), "second".to_string());
        env.insert("123".to_string(), "numeric".to_string());

        let replacer = VariableReplacer::new(&env);

        assert_eq!(replacer.replace("{{var1}}"), "first");
        assert_eq!(replacer.replace("{{var2}}"), "second");
        assert_eq!(replacer.replace("{{123}}"), "numeric");
    }

    #[test]
    fn test_variable_case_sensitivity() {
        let mut env = Environment::new();
        env.insert("Variable".to_string(), "uppercase".to_string());
        env.insert("variable".to_string(), "lowercase".to_string());
        env.insert("VARIABLE".to_string(), "allcaps".to_string());

        let replacer = VariableReplacer::new(&env);

        assert_eq!(replacer.replace("{{Variable}}"), "uppercase");
        assert_eq!(replacer.replace("{{variable}}"), "lowercase");
        assert_eq!(replacer.replace("{{VARIABLE}}"), "allcaps");
    }

    #[test]
    fn test_whitespace_in_variable_names() {
        let mut env = Environment::new();
        env.insert("var".to_string(), "value".to_string());

        let replacer = VariableReplacer::new(&env);

        // 变量名中的空格应该被忽略或导致不匹配
        assert_eq!(replacer.replace("{{ var }}"), "{{ var }}"); // 保持原样
        assert_eq!(replacer.replace("{{var }}"), "{{var }}"); // 保持原样
        assert_eq!(replacer.replace("{{ var}}"), "{{ var}}"); // 保持原样
    }

    #[test]
    fn test_complex_url_with_multiple_variables() {
        let mut env = Environment::new();
        env.insert("protocol".to_string(), "https".to_string());
        env.insert("host".to_string(), "api.example.com".to_string());
        env.insert("version".to_string(), "v2".to_string());
        env.insert("resource".to_string(), "users".to_string());
        env.insert("id".to_string(), "123".to_string());

        let replacer = VariableReplacer::new(&env);

        let url_template = "{{protocol}}://{{host}}/{{version}}/{{resource}}/{{id}}";
        let result = replacer.replace(url_template);

        assert_eq!(result, "https://api.example.com/v2/users/123");
    }

    #[test]
    fn test_variable_replacement_performance() {
        let mut env = Environment::new();
        for i in 0..100 {
            env.insert(format!("var{}", i), format!("value{}", i));
        }

        let replacer = VariableReplacer::new(&env);

        // 测试大量变量替换的性能
        let mut template = String::new();
        for i in 0..100 {
            template.push_str(&format!("{{{{var{}}}}}", i));
            if i < 99 {
                template.push(',');
            }
        }

        let result = replacer.replace(&template);

        // 验证所有变量都被正确替换
        for i in 0..100 {
            assert!(result.contains(&format!("value{}", i)));
        }
        assert!(!result.contains("{{"));
        assert!(!result.contains("}}"));
    }
}
