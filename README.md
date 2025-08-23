# HTML API Reader

A high-performance REST API server written in Rust that fetches and extracts content from web pages. Built using Clean Architecture principles with a workspace structure.

## Table of Contents

- [Quick Start](#quick-start)
- [Features](#features)
- [API Endpoints](#api-endpoints)
- [Architecture](#architecture)
- [Building](#building)
- [Running](#running)
- [Docker Setup](#docker-setup)
- [Project Structure](#project-structure)
- [Development](#development)
- [Error Handling](#error-handling)

## Quick Start

### 🚀 Using Local Development

1. **Build and run:**
   ```bash
   cargo build --release
   cargo run --bin html-mcp-reader
   ```

2. **Test the API:**
   ```bash
   # Health check
   curl http://localhost:8085/health
   
   # Fetch web content
   curl -X POST http://localhost:8085/api/fetch \
     -H "Content-Type: application/json" \
     -d '{"url": "https://example.com"}'
   ```

### 🐳 Using Docker

1. **Build the Docker image:**
   ```bash
   docker build -t html-api-reader:latest .
   ```

2. **Run the container:**
   ```bash
   docker run -p 8085:8085 html-api-reader:latest
   ```

3. **Or use Docker Compose:**
   ```bash
   docker-compose up
   ```

## Features

- **REST API**: Simple HTTP endpoints for web content fetching
- **HTML Content Extraction**: Extract text content from HTML pages
- **Flexible Options**: Configure text extraction, redirects, timeouts, and user agents
- **Clean Architecture**: Separated concerns with domain-driven design
- **Async/Await**: High-performance async processing with Tokio
- **CORS Support**: Cross-origin requests enabled
- **Health Monitoring**: Built-in health check endpoint
- **Docker Ready**: Containerized deployment with health checks

## API Endpoints

### GET /health

Returns the health status of the API server.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

### POST /api/fetch

Fetches and extracts content from web pages.

**Request Body:**
```json
{
  "url": "https://example.com",
  "extract_text_only": true,
  "follow_redirects": true,
  "timeout_seconds": 30,
  "user_agent": "html-api-reader/0.1.0"
}
```

**Parameters:**
- `url` (required): The URL to fetch content from
- `extract_text_only` (optional, default: true): Whether to extract only text content
- `follow_redirects` (optional, default: true): Whether to follow HTTP redirects
- `timeout_seconds` (optional, default: 30, max: 300): Request timeout in seconds
- `user_agent` (optional): Custom User-Agent header

**Response:**
```json
{
  "url": "https://example.com",
  "title": "Example Domain",
  "text_content": "Example Domain This domain is for use in illustrative examples...",
  "raw_html": "<!doctype html><html>...",
  "metadata": {
    "content_type": "text/html; charset=utf-8",
    "status_code": 200,
    "content_length": 1256,
    "last_modified": null,
    "charset": null
  }
}
```

**Error Response:**
```json
{
  "error": "INVALID_URL",
  "message": "URL cannot be empty"
}
```

## Architecture

The project follows Clean Architecture principles with these layers:

- **Domain**: Core business logic and interfaces (`domain/`)
- **Application**: Use cases and business services (`application/`)
- **Infrastructure**: External adapters for HTTP, HTML parsing, and REST API (`infrastructure/`)
- **Runner**: Entry point and dependency injection (`runner/`)

## Dependencies

Key dependencies used:
- `axum`: Modern web framework for the REST API
- `tower-http`: HTTP middleware (CORS support)
- `reqwest`: HTTP client for fetching web content
- `scraper`: HTML parsing and text extraction
- `serde`/`serde_json`: JSON serialization for API requests/responses
- `tracing`: Structured logging
- `tokio`: Async runtime

## Building

### Local Development

```bash
# Build debug version
cargo build

# Build release version (optimized)
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt
```

### Docker

```bash
# Build the Docker image
docker build -t html-api-reader:latest .

# Build with Docker Compose
docker-compose build
```

## Running

### Local Development

```bash
# Run in development mode
cargo run --bin html-mcp-reader

# Run release version
./target/release/html-mcp-reader

# Run with custom port
PORT=9000 cargo run --bin html-mcp-reader
```

The server will start on `http://0.0.0.0:8085` by default.

### Docker

```bash
# Run with Docker (recommended for production)
docker run -p 8085:8085 html-api-reader:latest

# Run with Docker Compose
docker-compose up

# Run in background
docker-compose up -d

# View logs
docker-compose logs -f html-api-reader

# Stop
docker-compose down
```

### Environment Variables

- `PORT`: Server port (default: 8085)
- `RUST_LOG`: Log level (default: info)
- `RUST_BACKTRACE`: Enable backtraces (default: 1)

## Docker Setup

### Prerequisites

- Docker installed on your system
- Docker Compose (usually included with Docker Desktop)

### Configuration

The `docker-compose.yaml` includes:
- **Port Mapping**: `8085:8085`
- **Health Check**: `curl http://localhost:8085/health`
- **Resource Limits**: 512M memory, 0.5 CPU
- **Auto-restart**: `unless-stopped`
- **Logging**: Structured logs with tracing

### Testing with Docker

```bash
# Build and start
docker-compose up --build

# Test health endpoint
curl http://localhost:8085/health

# Test content fetching
curl -X POST http://localhost:8085/api/fetch \
  -H "Content-Type: application/json" \
  -d '{"url": "https://httpbin.org/html"}'
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
│   │   └── api/            # REST API server implementation
└── runner/                # Application entry point
    └── src/
        └── main.rs         # Main application with DI setup
```

## Development

### Adding New Features

1. Define domain models in `domain/src/model/`
2. Create interfaces in `domain/src/port/`
3. Implement business logic in `application/src/service/` or `application/src/use_case/`
4. Create infrastructure adapters in `infrastructure/src/`
5. Wire dependencies in `runner/src/main.rs`

### API Development

To add new endpoints:

1. Add request/response models to `domain/src/model/request.rs`
2. Implement business logic in `application/src/use_case/`
3. Add route handlers in `infrastructure/src/api/server.rs`
4. Update the router in `create_router()` method

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific workspace member
cargo test -p domain
cargo test -p application
cargo test -p infrastructure

# Run specific test
cargo test test_name

# Integration tests with running server
cargo run --bin html-mcp-reader &
SERVER_PID=$!
curl http://localhost:8085/health
kill $SERVER_PID
```

## Error Handling

The API returns appropriate HTTP status codes and error responses:

### HTTP Status Codes
- `200 OK`: Successful request
- `400 Bad Request`: Invalid request parameters
- `500 Internal Server Error`: Server-side errors

### Error Response Format
```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable error description"
}
```

### Common Error Codes
- `INVALID_URL`: Empty or malformed URL
- `FETCH_ERROR`: Network, timeout, or HTTP errors
- `PARSE_ERROR`: HTML parsing failures

### Logging

The application uses structured logging with different levels:
- `INFO`: Normal operation logs
- `ERROR`: Error conditions
- `DEBUG`: Detailed debugging information

Configure logging with the `RUST_LOG` environment variable:
```bash
# Info level (default)
RUST_LOG=info cargo run

# Debug level for detailed logs
RUST_LOG=debug cargo run

# Module-specific logging
RUST_LOG=infrastructure::api::server=debug cargo run
```

## Performance

- **Async/Await**: Non-blocking I/O operations
- **Connection Pooling**: HTTP client reuses connections
- **Memory Efficient**: Streaming HTML parsing
- **Resource Limits**: Configurable timeouts and memory limits
- **Docker Optimization**: Multi-stage build with minimal runtime image

## Security

- **Non-root User**: Docker container runs as non-root user
- **Resource Limits**: Memory and CPU limits in Docker Compose
- **Input Validation**: URL validation and parameter sanitization
- **Timeout Protection**: Configurable request timeouts
- **CORS**: Cross-origin request support (configurable)

## License

This project is open source and available under the [MIT License](LICENSE).