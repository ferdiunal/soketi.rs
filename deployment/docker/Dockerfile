# Multi-stage build for Soketi Rust Server
# Docker Hub: funal/soketi-rs
# GitHub: https://github.com/ferdiunal/soketi.rs

# Stage 1: Builder
FROM rust:1.93-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Stage 2: Runtime — distroless (no shell, no package manager, minimal CVE surface)
# Kullanıcı: nonroot (UID 65532), glibc dahil, ~20 MB base
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /app/target/release/soketi-rs /usr/local/bin/soketi-rs
COPY --chown=65532:65532 deployment/docker/config.example.json /app/config.json

EXPOSE 6001
EXPOSE 9601

# HEALTHCHECK burada kullanılamaz (shell yok); K8s liveness/readiness probe kullan:
# livenessProbe:  httpGet: { path: /, port: 6001 }
# readinessProbe: httpGet: { path: /ready, port: 6001 }

ENTRYPOINT ["/usr/local/bin/soketi-rs"]
CMD ["--config-file", "/app/config.json"]
