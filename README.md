# HTML MCP Reader

A custom MCP (Model Context Protocol) server written in Rust that reads HTML or text content from web pages. Built using Clean Architecture principles with a workspace structure.

## Table of Contents

- [Quick Start](#quick-start)
- [Features](#features)
- [Architecture](#architecture)
- [Building](#building)
- [Running](#running)
- [MCP Tools](#mcp-tools)
- [Docker Setup and Configuration](#docker-setup-and-configuration)
- [MCP Client Configuration](#mcp-client-configuration)
- [Project Structure](#project-structure)
- [Development](#development)
- [Error Handling](#error-handling)

## Quick Start

### 🐳 Using Docker (Recommended)

1. **Build the Docker image:**
   ```bash
   docker compose build
   ```

2. **Test the MCP server:**
   ```bash
   echo '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{}}' | docker run -i --rm html-mcp-reader:latest
   ```

3. **Configure your MCP client:**
   ```json
   {
     "mcpServers": {
       "html-mcp-reader": {
         "command": "docker",
         "args": ["run", "-i", "--rm", "html-mcp-reader:latest"]
       }
     }
   }
   ```

### 🦀 Using Local Rust Binary

1. **Build and run:**
   ```bash
   cargo build
   cargo run --bin html-mcp-reader
   ```

2. **Configure your MCP client:**
   ```json
   {
     "mcpServers": {
       "html-mcp-reader": {
         "command": "cargo",
         "args": ["run", "--bin", "html-mcp-reader"],
         "cwd": "/path/to/your/html-mcp-reader"
       }
     }
   }
   ```

## Features

- Fetch content from web URLs
- Extract text content from HTML
- Parse HTML with customizable options
- MCP protocol compliant for AI assistant integration
- Clean Architecture with separated concerns
- Async/await support with tokio
- Configurable timeouts and User-Agent headers
- Support for following redirects

## Architecture

The project follows Clean Architecture principles with these layers:

- **Domain**: Core business logic and interfaces (`domain/`)
- **Application**: Use cases and business services (`application/`)
- **Infrastructure**: External adapters for HTTP, HTML parsing, and MCP protocol (`infrastructure/`)
- **Runner**: Entry point and dependency injection (`runner/`)

## Dependencies

Key dependencies used:
- `reqwest`: HTTP client for fetching web content
- `scraper`: HTML parsing and text extraction
- `serde`/`serde_json`: Serialization for MCP protocol
- `tracing`: Structured logging
- `tokio`: Async runtime

## Building

### Local Development

```bash
cargo build
```

### Docker

```bash
# Build the Docker image
docker compose build

# Or build with Docker directly
docker build -t html-mcp-reader:latest .
```

## Running

### Local Development

```bash
cargo run --bin html-mcp-reader
```

### Docker

```bash
# Run with Docker Compose (recommended for development)
docker-compose up

# Run single command with Docker
echo '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{}}' | docker run -i --rm html-mcp-reader:latest

# Interactive mode
docker run -it --rm html-mcp-reader:latest
```

The server communicates via JSON-RPC over stdin/stdout following the MCP protocol.

## MCP Tools

### fetch_web_content

Fetches and extracts content from web pages.

**Parameters:**
- `url` (required): The URL to fetch content from
- `extract_text_only` (optional, default: true): Whether to extract only text content
- `follow_redirects` (optional, default: true): Whether to follow HTTP redirects
- `timeout_seconds` (optional, default: 30, max: 300): Request timeout in seconds
- `user_agent` (optional): Custom User-Agent header

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "tools/call",
  "params": {
    "name": "fetch_web_content",
    "arguments": {
      "url": "https://example.com",
      "extract_text_only": true,
      "timeout_seconds": 30
    }
  }
}
```

## Project Structure

```
├── Cargo.toml              # Workspace configuration
├── domain/                 # Core business logic
│   ├── src/
│   │   ├── model/          # Domain models (content, request, response)
│   │   └── port/           # Interfaces for external dependencies
├── application/            # Business logic and use cases
│   ├── src/
│   │   ├── service/        # Application services
│   │   └── use_case/       # Use case implementations
├── infrastructure/        # External adapters
│   ├── src/
│   │   ├── client/         # HTTP client implementation
│   │   ├── adapter/        # HTML parser adapter
│   │   └── mcp/            # MCP server implementation
└── runner/                # Application entry point
    └── src/
        └── main.rs         # Main application with DI setup
```

## Development

To add new features:

1. Define domain models in `domain/src/model/`
2. Create interfaces in `domain/src/port/`
3. Implement business logic in `application/src/service/` or `application/src/use_case/`
4. Create infrastructure adapters in `infrastructure/src/`
5. Wire dependencies in `runner/src/main.rs`

## Docker Setup and Configuration

### Prerequisites

- Docker installed on your system
- Docker Compose (usually included with Docker Desktop)

### Building the Docker Image

1. **Build using Docker Compose (recommended):**
   ```bash
   docker compose build
   ```

2. **Build using Docker directly:**
   ```bash
   docker build -t html-mcp-reader:latest .
   ```

### Running with Docker

1. **Single request testing:**
   ```bash
   # Test initialization
   echo '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{}}' | docker run -i --rm html-mcp-reader:latest
   
   # Test tools listing
   echo '{"jsonrpc":"2.0","id":"2","method":"tools/list","params":{}}' | docker run -i --rm html-mcp-reader:latest
   
   # Test web content fetching
   echo '{"jsonrpc":"2.0","id":"3","method":"tools/call","params":{"name":"fetch_web_content","arguments":{"url":"https://example.com","extract_text_only":true}}}' | docker run -i --rm html-mcp-reader:latest
   ```

2. **Using provided test scripts:**
   ```bash
   # Make scripts executable
   chmod +x test-docker.sh test-mcp.sh
   
   # Run Docker tests
   ./test-docker.sh
   ```

3. **Development with Docker Compose:**
   ```bash
   # Start in background
   docker-compose up -d
   
   # View logs
   docker-compose logs -f html-mcp-reader
   
   # Stop
   docker-compose down
   ```

## MCP Client Configuration

### Claude Code

Add this configuration to your Claude Code settings:

**For Local Binary:**
```json
{
  "mcpServers": {
    "html-mcp-reader": {
      "command": "cargo",
      "args": ["run", "--bin", "html-mcp-reader"],
      "cwd": "/path/to/your/html-mcp-reader"
    }
  }
}
```

**For Docker:**
```json
{
  "mcpServers": {
    "html-mcp-reader": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "html-mcp-reader:latest"]
    }
  }
}
```

### Windsurf

Add to your Windsurf configuration file (`.windsurf/settings.json`):

**For Local Binary:**
```json
{
  "mcp.servers": {
    "html-mcp-reader": {
      "command": "cargo",
      "args": ["run", "--bin", "html-mcp-reader"],
      "cwd": "/path/to/your/html-mcp-reader"
    }
  }
}
```

**For Docker:**
```json
{
  "mcp.servers": {
    "html-mcp-reader": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "html-mcp-reader:latest"]
    }
  }
}
```

### Cursor

Add to your Cursor settings (`.vscode/settings.json` or global settings):

**For Local Binary:**
```json
{
  "mcp.servers": {
    "html-mcp-reader": {
      "command": "cargo",
      "args": ["run", "--bin", "html-mcp-reader"],
      "cwd": "/path/to/your/html-mcp-reader"
    }
  }
}
```

**For Docker:**
```json
{
  "mcp.servers": {
    "html-mcp-reader": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "html-mcp-reader:latest"]
    }
  }
}
```

### Generic MCP Client Configuration

For any MCP-compatible client, use these command configurations:

**Local Binary:**
- **Command:** `cargo`
- **Args:** `["run", "--bin", "html-mcp-reader"]`
- **Working Directory:** `/path/to/your/html-mcp-reader`

**Docker:**
- **Command:** `docker`
- **Args:** `["run", "-i", "--rm", "html-mcp-reader:latest"]`

### Environment Variables (Optional)

You can configure the following environment variables:

```json
{
  "mcpServers": {
    "html-mcp-reader": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "html-mcp-reader:latest"],
      "env": {
        "RUST_LOG": "info",
        "RUST_BACKTRACE": "1"
      }
    }
  }
}
```

### Testing Your Configuration

After adding the configuration, test it with your MCP client:

1. **Initialize the server:**
   ```
   Ask your AI assistant: "Can you initialize the html-mcp-reader?"
   ```

2. **List available tools:**
   ```
   Ask: "What tools are available in html-mcp-reader?"
   ```

3. **Fetch web content:**
   ```
   Ask: "Can you fetch the content from https://example.com using html-mcp-reader?"
   ```

### Troubleshooting

- **Docker not found:** Ensure Docker is installed and in your PATH
- **Permission denied:** Make sure your user can run Docker commands
- **Container startup issues:** Check logs with `docker-compose logs`
- **Network issues:** Ensure the container can access the internet for fetching web content

For more detailed Docker information, see [DOCKER.md](DOCKER.md).

## Error Handling

The server returns appropriate MCP error codes:
- `-32700`: Parse error
- `-32601`: Method not found
- `-32602`: Invalid parameters
- `-32001`: Network error
- `-32002`: Timeout error
- `-32003`: HTTP error
- `-32004`: Parse error