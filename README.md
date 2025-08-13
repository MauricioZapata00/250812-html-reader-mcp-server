# HTML MCP Reader

A custom MCP (Model Context Protocol) server written in Rust that reads HTML or text content from web pages. Built using Clean Architecture principles with a workspace structure.

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

```bash
cargo build
```

## Running

```bash
cargo run --bin html-mcp-reader
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

## Error Handling

The server returns appropriate MCP error codes:
- `-32700`: Parse error
- `-32601`: Method not found
- `-32602`: Invalid parameters
- `-32001`: Network error
- `-32002`: Timeout error
- `-32003`: HTTP error
- `-32004`: Parse error