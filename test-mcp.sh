#!/bin/bash

# Test script for HTML MCP Reader
echo "Testing HTML MCP Reader Docker Container..."

# Test 1: Initialize
echo "Test 1: Initialize"
echo '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{}}' | docker run -i --rm html-mcp-reader:latest

echo -e "\n"

# Test 2: List tools  
echo "Test 2: List Tools"
echo '{"jsonrpc":"2.0","id":"2","method":"tools/list","params":{}}' | docker run -i --rm html-mcp-reader:latest

echo -e "\n"

# Test 3: Fetch content (example.com)
echo "Test 3: Fetch Content"
echo '{"jsonrpc":"2.0","id":"3","method":"tools/call","params":{"name":"fetch_web_content","arguments":{"url":"https://example.com","extract_text_only":true}}}' | docker run -i --rm html-mcp-reader:latest

echo -e "\nTests completed!"