# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

**Build and Run:**
```bash
cargo build                           # Build all workspace members
cargo run --bin html-mcp-reader       # Run the MCP server
cargo check                           # Quick compilation check
```

**Testing:**
```bash
cargo test                            # Run all tests in workspace
cargo test -p domain                  # Test specific workspace member
cargo test test_name                  # Run specific test
```

**Development:**
```bash
cargo clippy                          # Rust linting
cargo fmt                             # Code formatting
```

## Architecture Overview

This is a **Clean Architecture** Rust workspace implementing an MCP (Model Context Protocol) server for web content fetching. The architecture enforces strict dependency direction: Domain ← Application ← Infrastructure ← Runner.

### Workspace Structure

- **domain/**: Core business logic with zero external dependencies
  - `model/`: Domain entities (HtmlContent, FetchContentRequest, McpResponse)
  - `port/`: Trait definitions for external dependencies (ContentFetcher, ContentParser)

- **application/**: Business logic and use cases
  - `service/`: Business services that orchestrate domain operations
  - `use_case/`: Complete business workflows (FetchWebContentUseCase)

- **infrastructure/**: External adapters implementing domain ports
  - `client/http_client.rs`: HTTP client using reqwest
  - `adapter/html_parser_adapter.rs`: HTML parsing using scraper
  - `mcp/server.rs`: MCP protocol JSON-RPC server

- **runner/**: Dependency injection and application entry point
  - `main.rs`: Wires all dependencies using Arc<T> and starts stdin/stdout MCP server

### Key Design Patterns

**Dependency Injection**: All dependencies are constructed in `runner/src/main.rs` using `Arc<T>` for shared ownership across async contexts.

**Generic Services**: Application services are generic over trait implementations:
```rust
ContentFetchService<F: ContentFetcher>
FetchWebContentUseCase<F: ContentFetcher, P: ContentParser>
```

**Error Handling**: Domain-specific error types using `thiserror`:
- `ContentFetcherError`: Network, HTTP, timeout errors
- `ContentParserError`: HTML parsing errors
- MCP errors mapped to JSON-RPC error codes

**Async Traits**: All external I/O operations use `#[async_trait]` for async trait methods.

## MCP Protocol Implementation

The server implements MCP 2024-11-05 protocol spec:
- Communicates via JSON-RPC over stdin/stdout
- Implements `tools/list`, `tools/call`, and `initialize` methods
- Single tool: `fetch_web_content` for web scraping

## Development Workflow

When adding new functionality:

1. **Define domain models** in `domain/src/model/` (pure data structures)
2. **Create trait interfaces** in `domain/src/port/` for external dependencies
3. **Implement business logic** in `application/src/service/` or `application/src/use_case/`
4. **Create infrastructure adapters** in `infrastructure/src/` implementing domain traits
5. **Wire dependencies** in `runner/src/main.rs` AppState::new()

The dependency flow ensures business logic never depends on external libraries directly.