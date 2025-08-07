//! HTTP解析模块
//!
//! 负责解析.http文件格式，提取HTTP请求信息。

use crate::SUPPORTED_METHODS;
use crate::error::{HttpieError, Result};
use crate::models::{Environment, HttpRequest};
use crate::variable::VariableReplacer;
use reqwest::Method;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;

/// HTTP解析器
#[derive(Debug)]
pub struct HttpParser {
    environment: Environment,
}

impl HttpParser {
    /// 创建新的HTTP解析器
    pub fn new(environment: Environment) -> Self {
        Self { environment }
    }

    /// 解析HTTP文件
    pub fn parse_file(&mut self, file_path: &str) -> Result<Vec<HttpRequest>> {
        let content = fs::read_to_string(file_path)
            .map_err(|_| HttpieError::FileNotFound(file_path.to_string()))?;

        // 解析文件内变量
        self.parse_file_variables(&content);

        // 解析请求
        self.parse_requests(&content)
    }

    /// 解析文件内变量定义
    fn parse_file_variables(&mut self, content: &str) {
        let mut file_variables = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('@') {
                if let Some(eq_pos) = line.find('=') {
                    let key = line[1..eq_pos].trim().to_string();
                    let value = line[eq_pos + 1..].trim().to_string();
                    file_variables.insert(key, value);
                }
            }
        }

        self.environment.extend(file_variables);
    }

    /// 解析HTTP请求
    fn parse_requests(&self, content: &str) -> Result<Vec<HttpRequest>> {
        let mut requests = Vec::new();
        let sections = self.split_into_sections(content);

        for section in sections {
            if let Some(request) = self.parse_request(&section)? {
                requests.push(request);
            }
        }

        Ok(requests)
    }

    /// 将内容分割为请求段落
    fn split_into_sections(&self, content: &str) -> Vec<String> {
        let mut sections = Vec::new();
        let mut current_section = String::new();
        let mut in_request = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // 跳过变量定义，但不跳过注释（因为###也是注释）
            if trimmed.starts_with('@') {
                continue;
            }

            // 检查是否是新的请求开始
            if trimmed.starts_with("###") {
                if in_request && !current_section.trim().is_empty() {
                    sections.push(current_section.clone());
                }
                current_section = String::new();
                in_request = true;
                current_section.push_str(line);
                current_section.push('\n');
            } else if in_request {
                current_section.push_str(line);
                current_section.push('\n');
            }
        }

        if in_request && !current_section.trim().is_empty() {
            sections.push(current_section);
        }
        sections
    }

    /// 解析单个请求
    fn parse_request(&self, section: &str) -> Result<Option<HttpRequest>> {
        let lines: Vec<&str> = section.lines().collect();
        if lines.is_empty() {
            return Ok(None);
        }

        let replacer = VariableReplacer::new(&self.environment);

        // 解析请求名称
        let name_line = lines[0].trim();
        if !name_line.starts_with("###") {
            return Ok(None);
        }
        let name = name_line[3..].trim().to_string();

        // 查找请求行
        let mut request_line_idx = None;
        for (i, line) in lines.iter().enumerate().skip(1) {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                // 检查是否包含HTTP方法
                for &method in SUPPORTED_METHODS {
                    if trimmed.starts_with(method) {
                        request_line_idx = Some(i);
                        break;
                    }
                }
                if request_line_idx.is_some() {
                    break;
                }
            }
        }

        let request_line_idx = match request_line_idx {
            Some(idx) => idx,
            None => return Ok(None),
        };

        // 解析请求行
        let request_line = replacer.replace(lines[request_line_idx].trim());
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(HttpieError::InvalidRequest(
                "Invalid request line format".to_string(),
            ));
        }

        let method = Method::from_str(parts[0])
            .map_err(|_| HttpieError::InvalidMethod(parts[0].to_string()))?;
        let url = parts[1].to_string();

        // 解析请求头
        let mut headers = HashMap::new();
        let mut body_start_idx = None;

        for (i, line) in lines.iter().enumerate().skip(request_line_idx + 1) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                body_start_idx = Some(i + 1);
                break;
            }

            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim().to_string();
                let value = replacer.replace(trimmed[colon_pos + 1..].trim());
                headers.insert(key, value);
            }
        }

        // 解析请求体
        let body = if let Some(start_idx) = body_start_idx {
            let body_lines: Vec<&str> = lines.iter().skip(start_idx).copied().collect();
            if body_lines.is_empty() {
                None
            } else {
                let body_content = body_lines.join("\n").trim().to_string();
                if body_content.is_empty() {
                    None
                } else {
                    Some(replacer.replace(&body_content))
                }
            }
        } else {
            None
        };

        let request = HttpRequest::new(name, method, url)
            .with_headers(headers)
            .with_body(body);

        Ok(Some(request))
    }
}
