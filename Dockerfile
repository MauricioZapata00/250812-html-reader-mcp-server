# Runtime stage - use a minimal base image
FROM debian:trixie-slim

# Install runtime dependencies including Chrome for browser automation
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libc6 \
    curl \
    wget \
    gnupg \
    && wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | gpg --dearmor -o /usr/share/keyrings/google-chrome.gpg \
    && echo "deb [arch=amd64 signed-by=/usr/share/keyrings/google-chrome.gpg] http://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list \
    && apt-get update \
    && apt-get install -y google-chrome-stable \
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