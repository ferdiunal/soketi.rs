# Pre-built binary Docker image for Soketi Rust Server
# Docker Hub: funal/soketi-rs
# GitHub: https://github.com/ferdiunal/soketi.rs
#
# Binary CI'da statik musl olarak derlenir ve build context'e eklenir.
# Docker icinde Rust derlemesi yapilmaz -- build suresi ~2-3 dakika.
#
# K8s probe onerileri:
# livenessProbe:  httpGet: { path: /, port: 6001 }
# readinessProbe: httpGet: { path: /ready, port: 6001 }

# Statik wget: shell olmadan healthcheck icin (distroless uyumlu)
FROM busybox:musl AS busybox

FROM gcr.io/distroless/static-debian12:nonroot

ARG TARGETARCH
COPY --from=busybox /bin/wget /usr/local/bin/wget
COPY soketi-rs-${TARGETARCH} /usr/local/bin/soketi-rs
COPY --chown=65532:65532 deployment/docker/config.example.json /app/config.json

EXPOSE 6001
EXPOSE 9601

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["/usr/local/bin/wget", "-qO", "/dev/null", "http://localhost:6001/"]

ENTRYPOINT ["/usr/local/bin/soketi-rs"]
CMD ["--config-file", "/app/config.json"]
