# =============================================================================
# caro - Natural Language to Shell Commands
# Multi-stage Docker build for minimal container image
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Builder
# Build the Rust binary with optimizations
# -----------------------------------------------------------------------------
FROM rust:1.83-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to build dependencies
RUN mkdir -p src && \
    echo 'fn main() { println!("dummy"); }' > src/main.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release --no-default-features --features remote-backends && \
    rm -rf src target/release/deps/caro*

# Copy actual source code
COPY src/ src/
COPY tests/ tests/
COPY benches/ benches/

# Build the actual application
RUN cargo build --release --no-default-features --features remote-backends

# Verify the binary works
RUN ./target/release/caro --version

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# Minimal runtime image with just the binary
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1000 caro

# Copy the built binary
COPY --from=builder /app/target/release/caro /usr/local/bin/caro

# Set permissions
RUN chmod +x /usr/local/bin/caro

# Switch to non-root user
USER caro
WORKDIR /home/caro

# Set environment variables
ENV CARO_CONFIG_DIR=/home/caro/.config/caro
ENV CARO_CACHE_DIR=/home/caro/.cache/caro

# Create config and cache directories
RUN mkdir -p $CARO_CONFIG_DIR $CARO_CACHE_DIR

# Default command shows help
ENTRYPOINT ["caro"]
CMD ["--help"]

# -----------------------------------------------------------------------------
# Metadata
# -----------------------------------------------------------------------------
LABEL org.opencontainers.image.title="caro"
LABEL org.opencontainers.image.description="Convert natural language to shell commands using local LLMs"
LABEL org.opencontainers.image.url="https://github.com/wildcard/caro"
LABEL org.opencontainers.image.source="https://github.com/wildcard/caro"
LABEL org.opencontainers.image.vendor="wildcard"
LABEL org.opencontainers.image.licenses="AGPL-3.0"
