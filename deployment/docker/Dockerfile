# Multi-stage build for Soketi Rust Server
# Docker Hub: funal/soketi-rs
# GitHub: https://github.com/ferdiunal/soketi.rs

# Stage 1: Builder
FROM rust:1.93-bookworm as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code (only src directory needed for build)
COPY src ./src

# Build release binary
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && apt-get install -y --no-install-recommends --only-upgrade \
    libssl3 \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 soketi

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/soketi-rs /usr/local/bin/soketi-rs

# Copy default config
COPY config.json /app/config.json

# Change ownership
RUN chown -R soketi:soketi /app

# Switch to non-root user
USER soketi

# Expose ports
EXPOSE 6001
EXPOSE 9601

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/bin/sh", "-c", "test -f /tmp/soketi.pid || exit 1"]

# Run the application
ENTRYPOINT ["soketi-rs"]
CMD ["--config-file", "/app/config.json"]
