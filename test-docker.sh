#!/bin/bash

# Test script for HTML MCP Reader Docker Container
echo "Testing HTML MCP Reader Docker Container..."

# Function to test a single MCP command
test_mcp_command() {
    local test_name="$1"
    local json_request="$2"
    
    echo "=== $test_name ==="
    echo "Request: $json_request"
    echo "Response:"
    echo "$json_request" | docker run -i --rm html-mcp-reader:latest
    echo ""
}

# Test 1: Initialize
test_mcp_command "Initialize" '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{}}'

# Test 2: List tools  
test_mcp_command "List Tools" '{"jsonrpc":"2.0","id":"2","method":"tools/list","params":{}}'

# Test 3: Unknown method
test_mcp_command "Unknown Method" '{"jsonrpc":"2.0","id":"3","method":"unknown/method","params":{}}'

echo "All tests completed!"
echo ""
echo "To test with a real web request, run:"
echo 'echo '"'"'{"jsonrpc":"2.0","id":"4","method":"tools/call","params":{"name":"fetch_web_content","arguments":{"url":"https://example.com","extract_text_only":true}}}'"'"' | docker run -i --rm html-mcp-reader:latest'