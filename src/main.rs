use clap::{Arg, Command};
use std::path::Path;
use tracing::{error, info};

use httpie::{
    DEFAULT_ENV_FILE, DEFAULT_HTTP_FILE, Environment, HttpClient, HttpParser, HttpRequest,
    HttpieError,
};

#[tokio::main]
async fn main() -> Result<(), HttpieError> {
    tracing_subscriber::fmt::init();

    let matches = Command::new("httpie")
        .version("0.1.0")
        .about("A simple HTTP client that parses .http files")
        .arg(
            Arg::new("file")
                .long("file")
                .value_name("FILE")
                .help("HTTP request definition file")
                .default_value(DEFAULT_HTTP_FILE),
        )
        .arg(
            Arg::new("case")
                .long("case")
                .value_name("CASE")
                .help("Specific test case to execute"),
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file").unwrap();
    let case_name = matches.get_one::<String>("case");

    // 尝试加载环境变量文件
    let env_file = Path::new(DEFAULT_ENV_FILE);
    let environment = if env_file.exists() {
        Environment::from_file(&env_file.to_string_lossy()).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load environment file: {e}");
            Environment::new()
        })
    } else {
        eprintln!(
            "Warning: Environment file '{}' not found, using empty environment",
            env_file.display()
        );
        Environment::new()
    };

    let mut parser = HttpParser::new(environment);

    let requests = parser.parse_file(file_path)?;

    if requests.is_empty() {
        info!("No valid HTTP requests found in file: {}", file_path);
        return Ok(());
    }

    info!("Found {} request(s) in file", requests.len());

    // 创建HTTP客户端并启用脚本功能
    let mut client = HttpClient::default().with_script_engine()?;

    // 执行请求
    match case_name {
        Some(case) => execute_specific_case(&mut client, &requests, case, file_path).await?,
        None => execute_all_requests(&mut client, &requests).await?,
    }

    Ok(())
}

/// 执行指定的测试用例
async fn execute_specific_case(
    client: &mut HttpClient,
    requests: &[HttpRequest],
    case_name: &str,
    _file_path: &str,
) -> Result<(), HttpieError> {
    // 查找指定的测试用例

    match requests.iter().find(|r| r.name.contains(case_name)) {
        Some(request) => {
            eprintln!("Found matching case: '{}'", request.name);
            eprintln!("Executing request to: {}", request.url);
            client.execute(request).await
        }
        None => {
            // 用例未找到
            Err(HttpieError::InvalidRequest(format!(
                "Case '{case_name}' not found"
            )))
        }
    }
}

/// 执行所有请求
async fn execute_all_requests(
    client: &mut HttpClient,
    requests: &[HttpRequest],
) -> Result<(), HttpieError> {
    info!("Executing all {} request(s)", requests.len());

    for (index, request) in requests.iter().enumerate() {
        info!(
            "Executing request {}/{}: {}",
            index + 1,
            requests.len(),
            request.name
        );

        if let Err(e) = client.execute(request).await {
            error!("Failed to execute request '{}': {}", request.name, e);
            return Err(e);
        }
    }

    Ok(())
}
