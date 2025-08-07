# HTTPie - Rust HTTP Client

一个用 Rust 实现的 HTTP 客户端工具，支持解析和执行 `.http` 文件格式的请求。

## 功能特性

- ✅ 解析 `.http` 文件格式的 HTTP 请求定义
- ✅ 支持多种 HTTP 方法（GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, GRAPHQL）
- ✅ 支持请求头和请求体
- ✅ 变量支持（文件内定义、环境变量文件、动态变量）
- ✅ 文件读取支持（使用 `< file_path` 语法）
- ✅ 环境配置支持（development/production 等）
- ✅ 命令行参数支持（--file, --case）
- ✅ JSON 响应格式化输出

## 安装和使用

### 编译项目

```bash
cargo build --release
```

### 基本使用

```bash
# 执行默认文件 ./test.http 中的所有请求
cargo run

# 指定 .http 文件
cargo run -- --file fixtures/test.http

# 执行特定的测试用例
cargo run -- --case "Test 1"

# 组合使用
cargo run -- --file test_variables.http --case "dynamic variables"
```

## .http 文件格式

### 基本语法

```http
### 请求名称
METHOD URL
Header-Name: Header-Value

{
  "request": "body"
}
```

### 变量支持

#### 1. 文件内变量定义

```http
@host = https://api.example.com
@token = my-secret-token

### 使用变量
GET {{host}}/users
Authorization: Bearer {{token}}
```

#### 2. 动态变量

```http
### 动态变量示例
POST {{host}}/data
X-Request-ID: {{$uuid}}
X-Timestamp: {{$timestamp}}
X-Random: {{$randomInt}}
X-Home: {{$processEnv.HOME}}
```

支持的动态变量：
- `{{$uuid}}` - 生成 UUID
- `{{$timestamp}}` - 当前时间戳（秒）
- `{{$randomInt}}` - 0-1000 的随机整数
- `{{$processEnv.VAR_NAME}}` - 系统环境变量

#### 3. 环境变量文件

创建 `http-client.env.json` 文件：

```json
{
  "development": {
    "host": "https://dev-api.example.com",
    "api_key": "dev-key-123"
  },
  "production": {
    "host": "https://api.example.com",
    "api_key": "prod-key-456"
  }
}
```

### 文件读取

```http
### 从文件读取请求体
POST {{host}}/upload
Content-Type: application/json

< ./data.json
```

## 示例文件

项目包含以下示例文件：

- `test.http` - 基本 HTTP 请求示例
- `test_variables.http` - 变量使用示例
- `test_env.http` - 环境变量示例
- `test_file_read.http` - 文件读取示例
- `http-client.env.json` - 环境配置示例

## 命令行选项

```
USAGE:
    httpie [OPTIONS]

OPTIONS:
        --file <FILE>    HTTP request definition file [default: ./test.http]
        --case <CASE>    Specific test case to execute
    -h, --help           Print help information
    -V, --version        Print version information
```

## 输出格式

工具会显示每个请求的详细信息：

```
=== Test 1: Basic GET request ===
Status: 200 OK
Headers:
  content-type: "application/json"
  content-length: "285"
Body:
{
  "args": {},
  "headers": {
    "Host": "httpbin.org"
  },
  "url": "https://httpbin.org/get"
}
```

## 依赖项

- `clap` - 命令行参数解析
- `reqwest` - HTTP 客户端
- `serde_json` - JSON 处理
- `tokio` - 异步运行时
- `regex` - 正则表达式
- `uuid` - UUID 生成
- `rand` - 随机数生成
- `tracing` - 日志记录
- `thiserror` - 错误处理

## 许可证

本项目采用 MIT 许可证。
