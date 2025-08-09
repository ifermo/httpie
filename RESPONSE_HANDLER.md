# Response Handler Scripts

这个功能允许你在HTTP请求后执行JavaScript脚本来处理响应数据，进行测试断言，以及管理全局变量。

## 语法

在.http文件中，你可以在请求定义后添加响应处理脚本：

```http
### 请求名称
GET https://httpbin.org/get

> {
%
// JavaScript代码
console.log('Status:', response.status);
test('Status should be 200', () => {
    expect(response.status).toBe(200);
});
%}
```

## 可用对象

### response 对象
- `response.status` - HTTP状态码
- `response.headers` - 响应头对象
- `response.body` - 响应体字符串
- `response.json()` - 将响应体解析为JSON对象

### client 对象
- `client.global` - 全局变量存储对象
- `client.global.set(key, value)` - 设置全局变量
- `client.global.get(key)` - 获取全局变量

### 测试函数
- `test(name, callback)` - 定义一个测试用例
- `expect(value)` - 创建断言对象
  - `.toBe(expected)` - 严格相等断言
  - `.toEqual(expected)` - 深度相等断言
  - `.toContain(substring)` - 包含断言
  - `.toBeGreaterThan(number)` - 大于断言
  - `.toBeLessThan(number)` - 小于断言

## 示例

### 基本测试
```http
### 测试GET请求
GET https://httpbin.org/get

> {
%
test('Status should be 200', () => {
    expect(response.status).toBe(200);
});

test('Should have correct headers', () => {
    expect(response.headers['content-type']).toContain('application/json');
});

const data = response.json();
test('Should have origin field', () => {
    expect(data.origin).toBeDefined();
});
%}
```

### 全局变量使用
```http
### 设置全局变量
GET https://httpbin.org/get

> {
%
// 保存响应数据到全局变量
const data = response.json();
client.global.set('test_url', data.url);
client.global.set('user_agent', data.headers['User-Agent']);
%}

### 使用全局变量
POST https://httpbin.org/post
Content-Type: application/json

{
  "previous_url": "{{test_url}}",
  "message": "Hello World"
}

> {
%
test('Should echo the previous URL', () => {
    const data = response.json();
    const previousUrl = client.global.get('test_url');
    expect(data.json.previous_url).toBe(previousUrl);
});
%}
```

### 错误处理
```http
### 测试404错误
GET https://httpbin.org/status/404

> {
%
test('Should handle 404 status', () => {
    expect(response.status).toBe(404);
});
%}
```

## 运行

使用以下命令运行包含响应处理脚本的HTTP文件：

```bash
cargo run -- --file your_file.http
```

测试结果会在响应输出之前显示，格式如下：

```
=== Test Results for 请求名称 ===
✓ PASS Test name
✗ FAIL Test name: Expected 200 but got 404
```

## 注意事项

1. 脚本使用Deno Core JavaScript引擎执行
2. 全局变量在整个会话中保持
3. 每个请求的脚本都在独立的作用域中执行
4. 支持现代JavaScript语法
5. 错误会被捕获并显示在测试结果中
