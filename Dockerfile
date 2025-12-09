# PMAT - Pragmatic Multi-language Agent Toolkit
# Multi-stage build for reproducible, minimal container image
#
# Build: docker build -t pmat .
# Run:   docker run -v $(pwd):/workspace pmat context /workspace

# Stage 1: Build environment
FROM rust:1.83-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/
COPY crates/pmat-dashboard/Cargo.toml ./crates/pmat-dashboard/

# Create dummy files to build dependencies
RUN mkdir -p server/src crates/pmat-dashboard/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "" > server/src/lib.rs && \
    echo "" > crates/pmat-dashboard/src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release --package pmat && \
    rm -rf server/src crates/pmat-dashboard/src

# Copy actual source
COPY server/src ./server/src
COPY server/benches ./server/benches
COPY server/templates ./server/templates
COPY crates ./crates

# Touch files to force rebuild
RUN touch server/src/main.rs server/src/lib.rs

# Build release binary
RUN cargo build --release --package pmat

# Stage 2: Runtime image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 pmat

# Copy binary from builder
COPY --from=builder /build/target/release/pmat /usr/local/bin/pmat

# Set ownership
RUN chown pmat:pmat /usr/local/bin/pmat

USER pmat
WORKDIR /workspace

# Verify installation
RUN pmat --version

# Default command
ENTRYPOINT ["pmat"]
CMD ["--help"]

# Labels for reproducibility
LABEL org.opencontainers.image.title="PMAT"
LABEL org.opencontainers.image.description="Pragmatic Multi-language Agent Toolkit"
LABEL org.opencontainers.image.source="https://github.com/paiml/paiml-mcp-agent-toolkit"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.version="2.210.0"
