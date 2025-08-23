# PMAT Docker Image - Multi-stage build for minimal production image
FROM rust:1.80-slim-bookworm as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libgit2-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r pmat && useradd -r -g pmat pmat

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Build the application
RUN cd server && \
    cargo build --release && \
    strip target/release/pmat

# Production image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libgit2-1.5 \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r pmat && useradd -r -g pmat pmat

# Copy binary from builder
COPY --from=builder /app/server/target/release/pmat /usr/local/bin/pmat

# Copy configuration files
COPY --from=builder /app/configs /etc/pmat/
COPY --from=builder /app/docs/CLAUDE_CODE_AGENT.md /usr/share/doc/pmat/

# Create directories
RUN mkdir -p /var/lib/pmat-agent /var/log/pmat-agent && \
    chown -R pmat:pmat /var/lib/pmat-agent /var/log/pmat-agent

# Set permissions
RUN chmod +x /usr/local/bin/pmat

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pmat --version || exit 1

# Labels for Docker Hub
LABEL org.opencontainers.image.title="PMAT"
LABEL org.opencontainers.image.description="Zero-config AI context generation and code quality toolkit with Claude Code Agent Mode"
LABEL org.opencontainers.image.url="https://github.com/paiml/paiml-mcp-agent-toolkit"
LABEL org.opencontainers.image.source="https://github.com/paiml/paiml-mcp-agent-toolkit"
LABEL org.opencontainers.image.version="2.10.0"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.authors="Pragmatic AI Labs <hello@paiml.com>"

# Default user
USER pmat

# Default command
CMD ["pmat", "--help"]

# Exposed ports (for web demo and metrics)
EXPOSE 8080 9090