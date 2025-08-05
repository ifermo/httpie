#!/usr/bin/env python3
"""
HTTPie - HTTP Request Executor
A Python script to parse and execute HTTP requests defined in .http files.

This module provides functionality to:
- Parse .http files containing HTTP request definitions
- Execute HTTP requests with proper headers and body handling
- Display formatted request and response information
"""

import argparse
import re
import sys
import urllib.request
from typing import Dict, List, Optional, Tuple


# 支持的HTTP方法常量
SUPPORTED_HTTP_METHODS = {"GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"}

# 默认配置
DEFAULT_CONFIG = {
    "default_file": "test.http",
    "encoding": "utf-8",
    "request_separator": "###",
}


class HttpRequest:
    """
    表示一个解析后的HTTP请求对象。

    包含HTTP请求的所有必要信息：请求名称、方法、URL、头部和请求体。
    """

    def __init__(
        self, name: str, method: str, url: str, headers: Dict[str, str], body: str
    ):
        """
        初始化HTTP请求对象。

        Args:
            name: 请求的名称标识
            method: HTTP方法（GET, POST等）
            url: 请求的URL地址
            headers: HTTP头部字典
            body: 请求体内容
        """
        self.name = name
        self.method = method.upper()
        self.url = url
        self.headers = headers
        self.body = body

    def __repr__(self) -> str:
        return (
            f"HttpRequest(name='{self.name}', method='{self.method}', url='{self.url}')"
        )

    def has_body(self) -> bool:
        """检查请求是否包含请求体。"""
        return bool(self.body.strip())

    def get_body_bytes(self) -> bytes:
        """获取请求体的字节表示。"""
        return self.body.encode(DEFAULT_CONFIG["encoding"]) if self.has_body() else b""


class HttpFileParser:
    """
    HTTP文件解析器，用于解析.http格式的文件。

    支持解析包含多个HTTP请求定义的文件，每个请求由###分隔。
    """

    def __init__(self, file_path: str):
        """
        初始化解析器。

        Args:
            file_path: .http文件的路径
        """
        self.file_path = file_path

    def parse(self) -> List[HttpRequest]:
        """
        解析.http文件并返回HTTP请求对象列表。

        Returns:
            解析成功的HttpRequest对象列表

        Raises:
            SystemExit: 当文件不存在或读取失败时退出程序
        """
        content = self._read_file()
        blocks = self._split_content(content)

        requests = []
        for block in blocks:
            block = block.strip()
            if not block:
                continue

            request = self._parse_block(block)
            if request:
                requests.append(request)

        return requests

    def _read_file(self) -> str:
        """
        读取文件内容。

        Returns:
            文件的文本内容

        Raises:
            SystemExit: 当文件读取失败时退出程序
        """
        try:
            with open(self.file_path, "r", encoding=DEFAULT_CONFIG["encoding"]) as f:
                return f.read()
        except FileNotFoundError:
            print(f"Error: File '{self.file_path}' not found.")
            sys.exit(1)
        except Exception as e:
            print(f"Error reading file: {e}")
            sys.exit(1)

    def _split_content(self, content: str) -> List[str]:
        """
        将文件内容按分隔符分割成请求块。

        Args:
            content: 文件内容

        Returns:
            分割后的请求块列表
        """
        return re.split(DEFAULT_CONFIG["request_separator"], content)

    def _parse_block(self, block: str) -> Optional[HttpRequest]:
        """
        解析单个HTTP请求块。

        Args:
            block: 单个请求块的文本内容

        Returns:
            解析成功的HttpRequest对象，失败时返回None
        """
        all_lines = [line.rstrip() for line in block.split("\n")]
        non_empty_lines = [line for line in all_lines if line.strip()]

        if not non_empty_lines:
            return None

        # 提取请求名称
        name = self._extract_request_name(non_empty_lines)

        # 查找并解析请求行
        request_line = self._find_request_line(non_empty_lines)
        if not request_line:
            return None

        method, url = self._parse_request_line(request_line)

        # 查找请求行在完整行列表中的位置
        request_line_index = self._find_request_line_index(all_lines, request_line)
        if request_line_index is None:
            return None

        # 解析头部和请求体
        headers, body = self._parse_headers_and_body(
            all_lines[request_line_index + 1 :]
        )

        return HttpRequest(name, method, url, headers, body)

    def _extract_request_name(self, non_empty_lines: List[str]) -> str:
        """
        从非空行列表中提取请求名称。

        Args:
            non_empty_lines: 非空行列表

        Returns:
            请求名称
        """
        return non_empty_lines[0].strip()

    def _find_request_line(self, non_empty_lines: List[str]) -> Optional[str]:
        """
        查找HTTP请求行（包含方法和URL）。

        Args:
            non_empty_lines: 非空行列表

        Returns:
            找到的请求行，未找到时返回None
        """
        if len(non_empty_lines) < 2:
            return None

        for line in non_empty_lines[1:]:
            line = line.strip()
            if line and not line.startswith("#"):
                parts = line.split(" ", 1)
                if len(parts) == 2 and parts[0].upper() in SUPPORTED_HTTP_METHODS:
                    return line
        return None

    def _parse_request_line(self, request_line: str) -> Tuple[str, str]:
        """
        解析请求行，提取HTTP方法和URL。

        Args:
            request_line: 请求行文本

        Returns:
            (方法, URL)的元组
        """
        parts = request_line.split(" ", 1)
        return parts[0].upper(), parts[1]

    def _find_request_line_index(
        self, all_lines: List[str], request_line: str
    ) -> Optional[int]:
        """
        在完整行列表中查找请求行的索引位置。

        Args:
            all_lines: 完整行列表（包含空行）
            request_line: 要查找的请求行

        Returns:
            请求行的索引位置，未找到时返回None
        """
        for i, line in enumerate(all_lines):
            if line.strip() == request_line:
                return i
        return None

    def _parse_headers_and_body(self, lines: List[str]) -> Tuple[Dict[str, str], str]:
        """
        解析HTTP头部和请求体。

        Args:
            lines: 请求行之后的所有行

        Returns:
            (头部字典, 请求体)的元组
        """
        headers = {}
        body_lines = []
        in_headers = True

        for line in lines:
            if in_headers:
                if not line.strip():  # 遇到空行，切换到请求体部分
                    in_headers = False
                    continue
                # 解析头部行
                if ":" in line:
                    key, value = line.split(":", 1)
                    headers[key.strip()] = value.strip()
                else:
                    # 非头部行但未遇到空行分隔符，为兼容性考虑将其视为请求体开始
                    in_headers = False
                    body_lines.append(line)
            else:
                # 请求体部分
                body_lines.append(line)

        body = "\n".join(body_lines).strip()
        return headers, body


class ResponseFormatter:
    """
    HTTP响应格式化器，负责格式化输出请求和响应信息。
    """

    @staticmethod
    def print_request_header(request: HttpRequest) -> None:
        """
        打印请求头部信息。

        Args:
            request: HTTP请求对象
        """
        print(f"\n===== {request.name} =====")
        print(f"{request.method} {request.url}")

    @staticmethod
    def print_request_headers(headers: Dict[str, str]) -> None:
        """
        打印请求头部字段。

        Args:
            headers: 头部字典
        """
        for key, value in headers.items():
            print(f"{key}: {value}")

    @staticmethod
    def print_request_body(body: str) -> None:
        """
        打印请求体。

        Args:
            body: 请求体内容
        """
        if body:
            print(f"\n{body}")

    @staticmethod
    def print_response_header(status: int, reason: str) -> None:
        """
        打印响应状态行。

        Args:
            status: HTTP状态码
            reason: 状态描述
        """
        print(f"\n----------")
        print(f"{status} {reason}")

    @staticmethod
    def print_response_headers(headers) -> None:
        """
        打印响应头部字段。

        Args:
            headers: 响应头部对象
        """
        for key, value in headers.items():
            print(f"{key}: {value}")

    @staticmethod
    def print_response_body(body: str) -> None:
        """
        打印响应体。

        Args:
            body: 响应体内容
        """
        print(f"\n{body}")

    @staticmethod
    def print_error(error_type: str, message: str, error_body: str = None) -> None:
        """
        打印错误信息。

        Args:
            error_type: 错误类型
            message: 错误消息
            error_body: 错误响应体（可选）
        """
        print(f"\n{error_type}: {message}")
        if error_body:
            print(f"Error Response: {error_body}")


class HttpExecutor:
    """
    HTTP请求执行器，负责执行HTTP请求并处理响应。
    """

    def __init__(self, formatter: ResponseFormatter = None):
        """
        初始化执行器。

        Args:
            formatter: 响应格式化器，默认使用ResponseFormatter
        """
        self.formatter = formatter or ResponseFormatter()

    def execute(self, request: HttpRequest) -> None:
        """
        执行单个HTTP请求。

        Args:
            request: 要执行的HTTP请求对象
        """
        self.formatter.print_request_header(request)

        # 准备请求
        headers, body_bytes = self._prepare_request(request)
        req = self._create_urllib_request(request, headers, body_bytes)

        # 打印请求详情
        self.formatter.print_request_headers(headers)
        self.formatter.print_request_body(request.body)

        # 执行请求
        self._execute_request(req)

    def _prepare_request(self, request: HttpRequest) -> Tuple[Dict[str, str], bytes]:
        """
        准备HTTP请求的头部和请求体。

        Args:
            request: HTTP请求对象

        Returns:
            (头部字典, 请求体字节)的元组
        """
        headers = request.headers.copy()
        body_bytes = request.get_body_bytes()

        # 如果有请求体，自动添加Content-Length头部
        if request.has_body():
            headers["Content-Length"] = str(len(body_bytes))

        return headers, body_bytes

    def _create_urllib_request(
        self, request: HttpRequest, headers: Dict[str, str], body_bytes: bytes
    ) -> urllib.request.Request:
        """
        创建urllib.request.Request对象。

        Args:
            request: HTTP请求对象
            headers: 头部字典
            body_bytes: 请求体字节

        Returns:
            配置好的urllib.request.Request对象
        """
        req = urllib.request.Request(request.url, data=body_bytes, headers=headers)
        req.get_method = lambda: request.method
        return req

    def _execute_request(self, req: urllib.request.Request) -> None:
        """
        执行HTTP请求并处理响应。

        Args:
            req: 配置好的urllib.request.Request对象
        """
        try:
            with urllib.request.urlopen(req) as response:
                self._handle_successful_response(response)
        except urllib.error.HTTPError as e:
            self._handle_http_error(e)
        except urllib.error.URLError as e:
            self._handle_url_error(e)
        except Exception as e:
            self._handle_general_error(e)

    def _handle_successful_response(self, response) -> None:
        """
        处理成功的HTTP响应。

        Args:
            response: HTTP响应对象
        """
        self.formatter.print_response_header(response.status, response.reason)
        self.formatter.print_response_headers(response.headers)

        response_body = response.read().decode(DEFAULT_CONFIG["encoding"])
        self.formatter.print_response_body(response_body)

    def _handle_http_error(self, error: urllib.error.HTTPError) -> None:
        """
        处理HTTP错误。

        Args:
            error: HTTP错误对象
        """
        error_body = None
        try:
            error_body = error.read().decode(DEFAULT_CONFIG["encoding"])
        except:
            pass

        self.formatter.print_error(
            "HTTP Error", f"{error.code} {error.reason}", error_body
        )

    def _handle_url_error(self, error: urllib.error.URLError) -> None:
        """
        处理URL错误。

        Args:
            error: URL错误对象
        """
        self.formatter.print_error("URL Error", str(error.reason))

    def _handle_general_error(self, error: Exception) -> None:
        """
        处理一般性错误。

        Args:
            error: 异常对象
        """
        self.formatter.print_error("Error", str(error))


class ArgumentParser:
    """
    命令行参数解析器，负责解析和处理命令行参数。
    """

    @staticmethod
    def create_parser() -> argparse.ArgumentParser:
        """
        创建命令行参数解析器。

        Returns:
            配置好的ArgumentParser对象
        """
        parser = argparse.ArgumentParser(
            description="Execute HTTP requests from .http files"
        )
        parser.add_argument(
            "--file",
            default=DEFAULT_CONFIG["default_file"],
            help=f"HTTP definition file (default: {DEFAULT_CONFIG['default_file']})",
        )
        parser.add_argument("--case", help="Specific request case to execute")
        return parser

    @staticmethod
    def resolve_file_path(file_arg: str) -> str:
        """
        解析文件路径参数。

        Args:
            file_arg: 命令行传入的文件参数

        Returns:
            解析后的文件路径
        """
        if file_arg == DEFAULT_CONFIG["default_file"] and not file_arg.startswith("/"):
            return f"./{file_arg}"
        return file_arg


class RequestFilter:
    """
    请求过滤器，负责根据条件过滤HTTP请求。
    """

    @staticmethod
    def filter_by_case(
        requests: List[HttpRequest], case_name: str
    ) -> List[HttpRequest]:
        """
        根据案例名称过滤请求。

        Args:
            requests: 所有HTTP请求列表
            case_name: 要匹配的案例名称

        Returns:
            过滤后的请求列表
        """
        return [r for r in requests if case_name.lower() in r.name.lower()]

    @staticmethod
    def print_available_requests(requests: List[HttpRequest]) -> None:
        """
        打印可用的请求列表。

        Args:
            requests: HTTP请求列表
        """
        print("Available requests:")
        for req in requests:
            print(f"  - {req.name}")


def parse_arguments() -> argparse.Namespace:
    """
    解析命令行参数。

    Returns:
        解析后的参数对象
    """
    parser = ArgumentParser.create_parser()
    return parser.parse_args()


def load_requests(file_path: str) -> List[HttpRequest]:
    """
    从文件加载HTTP请求。

    Args:
        file_path: HTTP文件路径

    Returns:
        HTTP请求列表
    """
    http_parser = HttpFileParser(file_path)
    return http_parser.parse()


def filter_requests(
    requests: List[HttpRequest], case_name: str = None
) -> List[HttpRequest]:
    """
    根据条件过滤请求。

    Args:
        requests: 所有HTTP请求列表
        case_name: 要匹配的案例名称（可选）

    Returns:
        过滤后的请求列表
    """
    if not case_name:
        return requests

    filtered_requests = RequestFilter.filter_by_case(requests, case_name)
    if not filtered_requests:
        print(f"No request found matching case: {case_name}")
        RequestFilter.print_available_requests(requests)
        return []

    return filtered_requests


def execute_requests(requests: List[HttpRequest]) -> None:
    """
    执行HTTP请求列表。

    Args:
        requests: 要执行的HTTP请求列表
    """
    executor = HttpExecutor()
    for request in requests:
        executor.execute(request)


def main() -> None:
    """
    主函数，程序入口点。

    负责协调整个程序的执行流程：
    1. 解析命令行参数
    2. 加载HTTP请求文件
    3. 过滤请求（如果指定了特定案例）
    4. 执行请求
    """
    # 解析命令行参数
    args = parse_arguments()

    # 解析文件路径
    file_path = ArgumentParser.resolve_file_path(args.file)

    # 加载HTTP请求
    requests = load_requests(file_path)

    if not requests:
        print("No valid HTTP requests found in file.")
        return

    # 过滤请求
    filtered_requests = filter_requests(requests, args.case)
    if not filtered_requests:
        return

    # 执行请求
    execute_requests(filtered_requests)


if __name__ == "__main__":
    main()
