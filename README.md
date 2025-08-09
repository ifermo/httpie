# HTTPie - A powerful HTTP client tool

一个功能强大的 HTTP 客户端工具，支持解析和执行 `.http` 文件格式的请求，具备完整的脚本处理和测试断言功能。

## 🚀 功能特性

### 核心功能
- ✅ **完整的 .http 文件解析** - 支持标准的 HTTP 文件格式
- ✅ **多种 HTTP 方法** - 支持 GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, GRAPHQL
- ✅ **请求头和请求体** - 完整的 HTTP 请求构建支持
- ✅ **响应格式化** - 自动格式化 JSON 响应，美化输出
- ✅ **文件读取** - 支持从文件读取请求体内容
- ✅ **命令行界面** - 灵活的命令行参数支持

### 变量系统
- ✅ **文件内变量定义** - 使用 `@variable = value` 语法
- ✅ **环境变量文件** - 支持多环境配置（development/production）
- ✅ **动态变量** - UUID、时间戳、随机数、系统环境变量
- ✅ **变量替换** - 使用 `{{variable}}` 语法进行变量替换

### 脚本和测试
- ✅ **JavaScript 响应处理器** - 基于 Deno Core 的脚本执行引擎
- ✅ **测试断言** - 内置测试框架，支持响应验证
- ✅ **全局变量** - 跨请求的变量共享
- ✅ **控制台输出** - 脚本调试支持

## 📦 安装和构建

### 系统要求
- Rust 1.70+ (使用 2024 edition)
- Cargo

### 编译项目

```bash
# 克隆项目
git clone <repository-url>
cd httpie

# 构建项目
cargo build --release

# 运行测试
cargo test
```

## 🔧 使用方法

### 基本命令

```bash
# 执行默认文件 ./test.http 中的所有请求
cargo run

# 指定 .http 文件
cargo run -- --file fixtures/test.http

# 执行特定的测试用例
cargo run -- --case "测试基本GET请求"

# 组合使用
cargo run -- --file fixtures/test_variables.http --case "dynamic variables"
```

### 命令行参数

- `--file <FILE>` - 指定 HTTP 请求定义文件（默认：`./test.http`）
- `--case <CASE>` - 执行特定的测试用例

## 📝 .http 文件格式

### 基本语法

```http
### 请求名称
METHOD URL
Header-Name: Header-Value
Another-Header: Another-Value

{
  "request": "body",
  "data": "value"
}
```

### 支持的 HTTP 方法

```http
### GET 请求
GET https://api.example.com/users

### POST 请求
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}

### PUT 请求
PUT https://api.example.com/users/1
Content-Type: application/json

{
  "name": "Jane Doe"
}

### DELETE 请求
DELETE https://api.example.com/users/1

### GraphQL 请求
GRAPHQL https://api.example.com/graphql
Content-Type: application/json

{
  "query": "{ users { id name email } }"
}
```

## 🔧 变量系统

### 1. 文件内变量定义

```http
@host = https://api.example.com
@token = my-secret-token
@version = v1

### 使用变量
GET {{host}}/{{version}}/users
Authorization: Bearer {{token}}
```

### 2. 环境变量文件

创建 `httpie.env.json` 文件：

```json
{
  "development": {
    "host": "https://dev-api.example.com",
    "api_key": "dev-key-123",
    "timeout": "30"
  },
  "production": {
    "host": "https://api.example.com",
    "api_key": "prod-key-456",
    "timeout": "60"
  }
}
```

在 .http 文件中使用：

```http
### 环境变量示例
GET {{host}}/api/data
X-API-Key: {{api_key}}
X-Timeout: {{timeout}}
```

### 3. 动态变量

```http
### 动态变量示例
POST {{host}}/data
X-Request-ID: {{$uuid}}
X-Timestamp: {{$timestamp}}
X-Random: {{$randomInt}}
X-Home: {{$processEnv.HOME}}

{
  "id": "{{$uuid}}",
  "timestamp": {{$timestamp}},
  "random": {{$randomInt}}
}
```

**支持的动态变量：**
- `{{$uuid}}` - 生成 UUID v4
- `{{$timestamp}}` - 当前时间戳（秒）
- `{{$randomInt}}` - 0-1000 的随机整数
- `{{$processEnv.VAR_NAME}}` - 系统环境变量

## 🧪 响应处理器和测试

### JavaScript 响应处理器

```http
### 带响应处理器的请求
GET {{host}}/api/users
Authorization: Bearer {{token}}

> {%
    // 测试状态码
    client.test("Status should be 200", function() {
        client.assert(response.status === 200, "Expected status 200, got " + response.status);
    });

    // 测试响应头
    client.test("Content-Type should be JSON", function() {
        client.assert(response.contentType.includes("application/json"), "Expected JSON content type");
    });

    // 测试响应体
    client.test("Response should contain users array", function() {
        client.assert(Array.isArray(response.body.users), "Users should be an array");
        client.assert(response.body.users.length > 0, "Users array should not be empty");
    });

    // 设置全局变量
    if (response.body.users.length > 0) {
        client.global.set("first_user_id", response.body.users[0].id);
    }

    // 控制台输出
    console.log("Found", response.body.users.length, "users");
%}
```

### 响应对象 API

在响应处理器脚本中，可以访问以下对象：

```javascript
// 响应对象
response.status        // HTTP 状态码
response.headers       // 响应头对象
response.body          // 响应体（自动解析 JSON）
response.contentType   // Content-Type 头

// 客户端对象
client.test(name, testFunction)     // 定义测试
client.assert(condition, message)   // 断言
client.global.set(key, value)       // 设置全局变量
client.global.get(key)              // 获取全局变量

// 控制台对象
console.log(...)       // 输出日志

// 全局断言函数
assert(condition, message)  // 全局断言函数
```

### 测试示例

```http
### 完整的测试示例
POST {{host}}/api/login
Content-Type: application/json

{
  "username": "admin",
  "password": "secret"
}

> {%
    client.test("Login should succeed", function() {
        client.assert(response.status === 200, "Login failed");
        client.assert(response.body.token !== undefined, "Token not provided");
    });

    client.test("Token should be valid format", function() {
        const token = response.body.token;
        client.assert(typeof token === "string", "Token should be string");
        client.assert(token.length > 10, "Token should be long enough");
    });

    // 保存 token 供后续请求使用
    client.global.set("auth_token", response.body.token);
%}

### 使用保存的 token
GET {{host}}/api/profile
Authorization: Bearer {{auth_token}}

> {%
    client.test("Profile access should work", function() {
        client.assert(response.status === 200, "Profile access failed");
        client.assert(response.body.user !== undefined, "User data not found");
    });
%}
```

## 🚀 示例用法

### 1. 基本 API 测试

```http
@host = https://jsonplaceholder.typicode.com

### 获取所有用户
GET {{host}}/users

> {%
    client.test("Should return users list", function() {
        client.assert(response.status === 200);
        client.assert(Array.isArray(response.body));
        client.assert(response.body.length > 0);
    });

    client.global.set("user_count", response.body.length);
%}

### 创建新用户
POST {{host}}/users
Content-Type: application/json

{
  "name": "Test User",
  "username": "testuser",
  "email": "test@example.com"
}

> {%
    client.test("Should create user successfully", function() {
        client.assert(response.status === 201);
        client.assert(response.body.id !== undefined);
    });
%}
```

### 2. 认证流程测试

```http
@api_base = https://api.example.com

### 登录获取 token
POST {{api_base}}/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "password123"
}

> {%
    client.test("Login should succeed", function() {
        client.assert(response.status === 200);
        client.assert(response.body.access_token !== undefined);
    });

    client.global.set("access_token", response.body.access_token);
%}

### 使用 token 访问受保护资源
GET {{api_base}}/user/profile
Authorization: Bearer {{access_token}}

> {%
    client.test("Should access protected resource", function() {
        client.assert(response.status === 200);
        client.assert(response.body.user !== undefined);
    });
%}
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🔗 相关链接

- [HTTP 文件格式规范](https://www.jetbrains.com/help/idea/http-client-in-product-code-editor.html)
- [Reqwest 文档](https://docs.rs/reqwest/)
- [Deno Core 文档](https://docs.rs/deno_core/)
- [Clap 文档](https://docs.rs/clap/)
