# Runtime stage - use a minimal base image
FROM debian:trixie-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libc6 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 appuser

# Set the working directory
WORKDIR /app

# Copy the working binary from host (ensures compatibility)
COPY target/release/html-mcp-reader /app/html-mcp-reader

# Change ownership to the non-root user
RUN chown -R appuser:appuser /app

# Switch to the non-root user
USER appuser

# Expose the port for the REST API
EXPOSE 8085

# Set environment variables
ENV RUST_LOG=info

# Run the application
ENTRYPOINT ["./html-mcp-reader"]