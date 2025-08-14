# Docker Setup for HTML MCP Reader

This document provides instructions for building and running the HTML MCP Reader application using Docker.

## Prerequisites

- Docker installed on your system
- Docker Compose (usually included with Docker Desktop)

## Building and Running

### Using Docker Compose (Recommended)

1. **Build and start the container:**
   ```bash
   docker-compose up --build
   ```

2. **Run in detached mode:**
   ```bash
   docker-compose up -d --build
   ```

3. **View logs:**
   ```bash
   docker-compose logs -f html-mcp-reader
   ```

4. **Stop the container:**
   ```bash
   docker-compose down
   ```

### Using Docker directly

1. **Build the image:**
   ```bash
   docker build -t html-mcp-reader:latest .
   ```

2. **Run the container:**
   ```bash
   docker run --name html-mcp-reader -it html-mcp-reader:latest
   ```

3. **Run in background:**
   ```bash
   docker run -d --name html-mcp-reader html-mcp-reader:latest
   ```

## Using the MCP Server

Since this is an MCP (Model Context Protocol) server, it communicates via stdin/stdout. Here are some ways to interact with it:

### Interactive Mode
```bash
# Start the container in interactive mode
docker-compose run --rm html-mcp-reader

# Or with docker directly
docker run -it --rm html-mcp-reader:latest
```

### Send MCP Commands

**Note**: The MCP server processes one JSON-RPC request at a time. Each docker run command will start a new container, process the request, and exit.

```bash
# Example: Initialize the server
echo '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{}}' | docker run -i --rm html-mcp-reader:latest

# Example: List available tools
echo '{"jsonrpc":"2.0","id":"2","method":"tools/list","params":{}}' | docker run -i --rm html-mcp-reader:latest

# Example: Fetch web content
echo '{"jsonrpc":"2.0","id":"3","method":"tools/call","params":{"name":"fetch_web_content","arguments":{"url":"https://example.com","extract_text_only":true}}}' | docker run -i --rm html-mcp-reader:latest

# Testing script
# Use the provided test scripts:
./test-docker.sh     # Tests basic MCP functionality
./test-mcp.sh        # Tests with local binary (for comparison)
```

## Configuration

### Environment Variables

The following environment variables can be configured in the `docker-compose.yaml`:

- `RUST_LOG`: Set log level (trace, debug, info, warn, error) - default: info
- `RUST_BACKTRACE`: Enable backtraces for debugging (0, 1, full) - default: 1

### Resource Limits

The docker-compose configuration includes resource limits:
- Memory: 512MB limit, 256MB reservation
- CPU: 0.5 cores limit, 0.25 cores reservation

You can adjust these in the `docker-compose.yaml` file as needed.

## Health Checks

The container includes a health check that verifies the process is running:
- Interval: 30 seconds
- Timeout: 10 seconds
- Retries: 3
- Start period: 40 seconds

## Development

### Rebuilding After Code Changes

```bash
# Rebuild and restart
docker-compose up --build

# Force rebuild without cache
docker-compose build --no-cache
docker-compose up
```

### Debugging

```bash
# Access container shell
docker-compose exec html-mcp-reader /bin/bash

# Or if container is not running
docker run -it --rm html-mcp-reader:latest /bin/bash
```

### Viewing Container Logs

```bash
# Follow logs
docker-compose logs -f

# View last 100 lines
docker-compose logs --tail=100

# View logs for specific time range
docker-compose logs --since="2024-01-01T00:00:00Z"
```

## Troubleshooting

### Container Won't Start
1. Check logs: `docker-compose logs html-mcp-reader`
2. Verify image was built successfully
3. Check resource availability

### High Memory Usage
1. Adjust memory limits in docker-compose.yaml
2. Monitor with: `docker stats html-mcp-reader`

### Network Issues
1. Ensure the container has internet access for fetching web content
2. Check firewall settings if needed

## Security Considerations

- The container runs as a non-root user (appuser)
- Only necessary runtime dependencies are included
- SSL certificates are included for HTTPS requests
- No sensitive data should be stored in the container

## Production Deployment

For production deployments, consider:

1. **Using specific image tags instead of `latest`**
2. **Setting up proper logging aggregation**
3. **Implementing container orchestration (Kubernetes, Docker Swarm)**
4. **Setting up monitoring and alerting**
5. **Using secrets management for sensitive configuration**