#!/usr/bin/env python3
"""
HTTPie - HTTP Request Executor
A Python script to parse and execute HTTP requests defined in .http files.
"""

import argparse
import re
import sys
import urllib.request
from typing import Dict, List, Optional


class HttpRequest:
    """Represents a parsed HTTP request."""

    def __init__(
        self, name: str, method: str, url: str, headers: Dict[str, str], body: str
    ):
        self.name = name
        self.method = method.upper()
        self.url = url
        self.headers = headers
        self.body = body

    def __repr__(self):
        return (
            f"HttpRequest(name='{self.name}', method='{self.method}', url='{self.url}')"
        )


class HttpFileParser:
    """Parser for .http files."""

    def __init__(self, file_path: str):
        self.file_path = file_path

    def parse(self) -> List[HttpRequest]:
        """Parse the .http file and return a list of HttpRequest objects."""
        try:
            with open(self.file_path, "r", encoding="utf-8") as f:
                content = f.read()
        except FileNotFoundError:
            print(f"Error: File '{self.file_path}' not found.")
            sys.exit(1)
        except Exception as e:
            print(f"Error reading file: {e}")
            sys.exit(1)

        requests = []
        blocks = re.split(r"###", content)

        for block in blocks:
            block = block.strip()
            if not block:
                continue

            request = self._parse_block(block)
            if request:
                requests.append(request)

        return requests

    def _parse_block(self, block: str) -> Optional[HttpRequest]:
        """Parse a single HTTP request block."""
        # Keep all lines including empty ones for proper header/body separation
        all_lines = [line.rstrip() for line in block.split("\n")]
        # Filter out completely empty lines only for finding name and request line
        non_empty_lines = [line for line in all_lines if line.strip()]

        if not non_empty_lines:
            return None

        # Extract name from first non-empty line
        name = non_empty_lines[0].strip()

        # Find the request line (should be method + URL)
        if len(non_empty_lines) < 2:
            return None

        request_line = None
        for line in non_empty_lines[1:]:
            line = line.strip()
            if line and not line.startswith("#"):
                parts = line.split(" ", 1)
                if len(parts) == 2 and parts[0].upper() in [
                    "GET",
                    "POST",
                    "PUT",
                    "DELETE",
                    "PATCH",
                    "HEAD",
                    "OPTIONS",
                ]:
                    request_line = line
                    break

        if not request_line:
            return None

        # Parse request line
        parts = request_line.split(" ", 1)
        method, url = parts[0].upper(), parts[1]

        # Find request line index in all_lines (including empty lines)
        request_line_index = None
        for i, line in enumerate(all_lines):
            if line.strip() == request_line:
                request_line_index = i
                break

        if request_line_index is None:
            return None

        # Parse headers and body with strict empty line separation
        headers = {}
        body_lines = []
        in_headers = True
        found_empty_line = False

        for line in all_lines[request_line_index + 1 :]:
            if in_headers:
                if not line.strip():  # Empty line found
                    found_empty_line = True
                    in_headers = False
                    continue
                # If we're still in headers, this line must be a header
                if ":" in line:
                    key, value = line.split(":", 1)
                    headers[key.strip()] = value.strip()
                else:
                    # Non-header line found before empty line separator - this is invalid
                    # But we'll treat it as the start of body for compatibility
                    in_headers = False
                    body_lines.append(line)
            else:
                # We're in body section
                body_lines.append(line)

        body = "\n".join(body_lines).strip()

        return HttpRequest(name, method, url, headers, body)


class HttpExecutor:
    """Executes HTTP requests."""

    def __init__(self):
        pass

    def execute(self, request: HttpRequest) -> None:
        """Execute a single HTTP request."""
        print(f"\n===== {request.name} =====")
        print(f"{request.method} {request.url}")

        # Prepare headers
        headers = request.headers.copy()

        # Calculate and add Content-Length if body exists
        if request.body:
            body_bytes = request.body.encode("utf-8")
            headers["Content-Length"] = str(len(body_bytes))
        else:
            body_bytes = b""

        # Create request
        req = urllib.request.Request(request.url, data=body_bytes, headers=headers)
        req.get_method = lambda: request.method

        # Print request details
        for key, value in headers.items():
            print(f"{key}: {value}")
        if request.body:
            print(f"\n{request.body}")

        try:
            # Execute request
            with urllib.request.urlopen(req) as response:
                print(f"\n----------")
                print(f"{response.status} {response.reason}")
                for key, value in response.headers.items():
                    print(f"{key}: {value}")

                # Read response body
                response_body = response.read().decode("utf-8")
                print(f"\n{response_body}")

        except urllib.error.HTTPError as e:
            print(f"\nHTTP Error: {e.code} {e.reason}")
            try:
                error_body = e.read().decode("utf-8")
                print(f"Error Response: {error_body}")
            except:
                pass
        except urllib.error.URLError as e:
            print(f"\nURL Error: {e.reason}")
        except Exception as e:
            print(f"\nError: {e}")


def main():
    parser = argparse.ArgumentParser(
        description="Execute HTTP requests from .http files"
    )
    parser.add_argument(
        "--file", default="test.http", help="HTTP definition file (default: test.http)"
    )
    parser.add_argument("--case", help="Specific request case to execute")

    args = parser.parse_args()

    # Default file path
    if args.file == "test.http" and not args.file.startswith("/"):
        file_path = "./test.http"
    else:
        file_path = args.file

    # Parse HTTP file
    http_parser = HttpFileParser(file_path)
    requests = http_parser.parse()

    if not requests:
        print("No valid HTTP requests found in file.")
        return

    # Filter by case if specified
    if args.case:
        filtered_requests = [r for r in requests if args.case.lower() in r.name.lower()]
        if not filtered_requests:
            print(f"No request found matching case: {args.case}")
            print("Available requests:")
            for req in requests:
                print(f"  - {req.name}")
            return
        requests = filtered_requests

    # Execute requests
    executor = HttpExecutor()
    for request in requests:
        executor.execute(request)


if __name__ == "__main__":
    main()
