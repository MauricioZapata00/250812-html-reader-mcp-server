#!/usr/bin/env python3
"""
Test script to demonstrate the browser automation functionality.
This tests the MCP server's ability to fetch JavaScript-rendered content.
"""

import json
import subprocess
import sys

def test_mcp_server():
    """Test the MCP server with browser automation."""
    
    # Test cases: Static content vs JavaScript-heavy content
    test_cases = [
        {
            "name": "Static HTML Test",
            "url": "https://httpbin.org/html",
            "description": "Should use static fetcher"
        },
        {
            "name": "JavaScript SPA Test", 
            "url": "https://jsonplaceholder.typicode.com/",
            "description": "Should detect and use browser fetcher"
        }
    ]
    
    # Start the MCP server
    print("🤖 Starting HTML MCP Reader with browser automation...")
    process = subprocess.Popen(
        ["cargo", "run", "--", "mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd="/home/mauricio/Documentos/MyDummyProjects/RustProjects/250812-html-mcp-reader/html-mcp-reader"
    )
    
    try:
        # Initialize the MCP server
        init_request = {
            "jsonrpc": "2.0",
            "id": "init",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        }
        
        print("📋 Sending initialization request...")
        process.stdin.write(json.dumps(init_request) + "\n")
        process.stdin.flush()
        
        init_response = process.stdout.readline()
        if init_response:
            print("✅ Server initialized successfully")
            print(f"📋 Response: {init_response.strip()}")
        
        # Test each URL
        for i, test_case in enumerate(test_cases, 1):
            print(f"\n🧪 Test {i}: {test_case['name']}")
            print(f"🔗 URL: {test_case['url']}")
            print(f"📝 {test_case['description']}")
            
            fetch_request = {
                "jsonrpc": "2.0",
                "id": f"test-{i}",
                "method": "tools/call",
                "params": {
                    "name": "fetch_web_content",
                    "arguments": {
                        "url": test_case["url"],
                        "timeout_seconds": 10
                    }
                }
            }
            
            print("📤 Sending fetch request...")
            process.stdin.write(json.dumps(fetch_request) + "\n")
            process.stdin.flush()
            
            response_line = process.stdout.readline()
            if response_line:
                try:
                    response = json.loads(response_line.strip())
                    if "result" in response:
                        content = response["result"]["content"]
                        metadata = content.get("metadata", {})
                        method = metadata.get("fetch_method", "Unknown")
                        js_detected = metadata.get("javascript_detected", None)
                        
                        print(f"✅ Fetch successful!")
                        print(f"🔍 Fetch method used: {method}")
                        print(f"🔍 JavaScript detected: {js_detected}")
                        print(f"📄 Content length: {len(content.get('text_content', ''))}")
                        print(f"📄 Title: {content.get('title', 'No title')}")
                    else:
                        print(f"❌ Error: {response.get('error', 'Unknown error')}")
                except json.JSONDecodeError:
                    print(f"❌ Invalid JSON response: {response_line}")
            else:
                print("❌ No response received")
    
    except KeyboardInterrupt:
        print("\n⚠️ Test interrupted by user")
    except Exception as e:
        print(f"❌ Test error: {e}")
    finally:
        print("\n🛑 Terminating server...")
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
        print("✅ Server terminated")

if __name__ == "__main__":
    print("🚀 Browser Automation Test for HTML MCP Reader")
    print("=" * 50)
    test_mcp_server()