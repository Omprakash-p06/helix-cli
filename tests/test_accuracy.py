#!/usr/bin/env python3
import sys
import json
import requests
import time

URL = "http://127.0.0.1:8080/v1/chat/completions"

tools = [
    {
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "Read the contents of a file",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Absolute file path"}
                },
                "required": ["path"]
            }
        }
    },
    {
        "type": "function",
        "function": {
            "name": "write_file",
            "description": "Write data to a file",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Absolute file path"},
                    "content": {"type": "string", "description": "Content to write"}
                },
                "required": ["path", "content"]
            }
        }
    }
]

tests = [
    ("Please read the file /etc/hosts", "read_file"),
    ("Write a python loop into C:\\temp\\hello.py", "write_file"),
    ("What is the capital of France?", None),
]

def main():
    print("="*55)
    print("  Helix Native Tool Calling Accuracy Test Benchmark")
    print("="*55)

    try:
        # Check if server is up
        requests.get("http://127.0.0.1:8080/v1/models", timeout=3)
    except Exception:
        print("[!] Backend server is not reachable at 127.0.0.1:8080. Start the server first.")
        sys.exit(1)

    passed = 0
    failed = 0

    for prompt, expected_tool in tests:
        print(f"\n[Test] Prompt: '{prompt}'")
        print(f"       Expected Tool: {expected_tool if expected_tool else 'None (Direct Answer)'}")
        
        payload = {
            "model": "local-model",
            "messages": [{"role": "user", "content": prompt}],
            "tools": tools,
            "temperature": 0.1
        }
        
        try:
            start_time = time.time()
            res = requests.post(URL, json=payload, timeout=120)
            duration = time.time() - start_time
            res.raise_for_status()
            data = res.json()
            
            message = data["choices"][0]["message"]
            tool_calls = message.get("tool_calls", [])
            
            if expected_tool:
                if tool_calls and len(tool_calls) > 0 and tool_calls[0]["function"]["name"] == expected_tool:
                    print(f"  [ ✓ ] Correctly invoked {expected_tool} in {duration:.2f}s")
                    try:
                        args = json.loads(tool_calls[0]["function"]["arguments"])
                        print(f"        Args: {args}")
                        passed += 1
                    except Exception as e:
                        print(f"  [ ✗ ] Tool invoked but arguments were invalid JSON: {e}")
                        failed += 1
                else:
                    print(f"  [ ✗ ] Expected {expected_tool}, got: {tool_calls}")
                    print(f"        Raw Content Fallback: {message.get('content', '')}")
                    failed += 1
            else:
                if not tool_calls:
                    print(f"  [ ✓ ] Correctly ignored tools and responded natively in {duration:.2f}s")
                    passed += 1
                else:
                    print(f"  [ ✗ ] Unexpectedly invoked a tool: {tool_calls}")
                    failed += 1
                    
        except Exception as e:
            print(f"  [ ✗ ] Request crashed: {e}")
            failed += 1

    print("\n" + "="*55)
    print(f"  Results: {passed} Passed, {failed} Failed")
    print("="*55)
    
    if failed > 0:
        sys.exit(1)
    sys.exit(0)

if __name__ == "__main__":
    main()
