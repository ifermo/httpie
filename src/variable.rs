//! 变量替换模块
//!
//! 处理HTTP请求中的各种变量替换，包括动态变量、环境变量和用户自定义变量。

use crate::models::Environment;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// 变量替换器
#[derive(Debug)]
pub struct VariableReplacer<'a> {
    environment: &'a Environment,
}

impl<'a> VariableReplacer<'a> {
    /// 创建新的变量替换器
    pub fn new(environment: &'a Environment) -> Self {
        Self { environment }
    }

    /// 替换文本中的所有变量
    pub fn replace(&self, text: &str) -> String {
        let mut result = text.to_string();

        // 替换动态变量
        result = self.replace_dynamic_variables(&result);

        // 替换环境变量
        result = self.replace_env_variables(&result);

        // 替换用户自定义变量
        result = self.replace_user_variables(&result);

        result
    }

    /// 替换动态变量（$uuid, $timestamp, $randomInt）
    fn replace_dynamic_variables(&self, text: &str) -> String {
        let mut result = text.to_string();

        // 替换 $uuid
        if result.contains("$uuid") {
            let uuid = Uuid::new_v4().to_string();
            result = result.replace("$uuid", &uuid);
        }

        // 替换 $timestamp
        if result.contains("$timestamp") {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string();
            result = result.replace("$timestamp", &timestamp);
        }

        // 替换 $randomInt
        if result.contains("$randomInt") {
            let random_int = rand::rng().random_range(1..=1000000).to_string();
            result = result.replace("$randomInt", &random_int);
        }

        result
    }

    /// 替换环境变量（$processEnv.VARIABLE_NAME）
    fn replace_env_variables(&self, text: &str) -> String {
        let mut result = text.to_string();

        // 查找所有 $processEnv. 模式
        while let Some(start) = result.find("$processEnv.") {
            let var_start = start + "$processEnv.".len();
            let var_end = result[var_start..]
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .map(|i| var_start + i)
                .unwrap_or(result.len());

            let var_name = &result[var_start..var_end];
            let replacement = std::env::var(var_name).unwrap_or_default();

            result.replace_range(start..var_end, &replacement);
        }

        result
    }

    /// 替换用户自定义变量（{{variable_name}}）
    fn replace_user_variables(&self, text: &str) -> String {
        let mut result = text.to_string();

        for (key, value) in self.environment.variables() {
            let pattern = format!("{{{{{key}}}}}");
            result = result.replace(&pattern, value);
        }

        result
    }
}
