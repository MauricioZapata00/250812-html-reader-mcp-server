# Use the specified Rust slim image
FROM rust:1.89.0-slim-trixie AS builder

# Install system dependencies needed for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files first to leverage Docker layer caching
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY domain/Cargo.toml domain/
COPY application/Cargo.toml application/
COPY infrastructure/Cargo.toml infrastructure/
COPY runner/Cargo.toml runner/

# Create dummy source files to build dependencies
RUN mkdir -p domain/src application/src infrastructure/src runner/src && \
    echo "fn main() {}" > runner/src/main.rs && \
    echo "// dummy" > domain/src/lib.rs && \
    echo "// dummy" > application/src/lib.rs && \
    echo "// dummy" > infrastructure/src/lib.rs

# Build dependencies to cache them
RUN cargo build --release --bin html-mcp-reader
RUN rm -rf domain/src application/src infrastructure/src runner/src

# Copy the actual source code
COPY domain domain
COPY application application
COPY infrastructure infrastructure
COPY runner runner

# Build the application
RUN cargo build --release --bin html-mcp-reader

# Runtime stage - use a minimal base image
FROM debian:trixie-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libc6 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 appuser

# Set the working directory
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/html-mcp-reader /app/html-mcp-reader

# Change ownership to the non-root user
RUN chown -R appuser:appuser /app

# Switch to the non-root user
USER appuser

# Expose the port if the application uses one (MCP typically uses stdin/stdout)
# EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info

# Run the application
CMD ["./html-mcp-reader"]