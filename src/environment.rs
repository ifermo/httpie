//! 环境配置模块
//!
//! 处理环境配置文件的加载和管理。

use crate::error::Result;
use crate::models::Environment;
use std::path::Path;

/// 环境配置加载器
pub struct EnvironmentLoader;

impl EnvironmentLoader {
    /// 从指定路径加载环境配置
    pub fn load_from_path(env_file: &str) -> Result<Environment> {
        if Path::new(env_file).exists() {
            Environment::from_file(env_file)
        } else {
            eprintln!("Warning: Environment file '{env_file}' not found, using empty environment");
            Ok(Environment::new())
        }
    }

    /// 从基础路径和环境文件名加载配置
    pub fn load_from_base_path(base_path: &Path, env_filename: &str) -> Result<Environment> {
        let env_file = base_path.join(env_filename);
        let env_file_str = env_file.to_string_lossy();
        Self::load_from_path(&env_file_str)
    }
}
