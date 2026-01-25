# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 1.0.0 (2026-01-25)

### 🐛 Bug Fixes

* Update Dockerfile path and license to GPL-3.0 ([00bca6c](https://github.com/ferdiunal/soketi.rs/commit/00bca6c968de67fe22aafd4d90ded2a7de7ea283))

### 📚 Documentation

* add Docker Hub README and update license information ([271eafd](https://github.com/ferdiunal/soketi.rs/commit/271eafd8bca175f5aae77cf05f1d04c8fe7ba8d0))
* add release and versioning documentation ([9609c8b](https://github.com/ferdiunal/soketi.rs/commit/9609c8bd92cd452928d7f2361d766478341f0f88))

### 👷 CI/CD

* **.github/workflows:** update Docker username in publish workflow ([07e0f1d](https://github.com/ferdiunal/soketi.rs/commit/07e0f1d62b4ea1fef2b4e519c6bfc7063aefc47a))

## [Unreleased]

## [0.1.0] - 2026-01-25

### Added
- Initial release of Soketi.rs
- High-performance WebSocket server written in Rust
- 100% Pusher protocol compatibility
- Support for public, private, and presence channels
- Multiple app manager backends (Array, MySQL, PostgreSQL, DynamoDB)
- Multiple adapter types (Local, Redis, NATS, Cluster)
- Cache managers (Memory, Redis)
- Rate limiting (Local, Redis)
- Queue managers (Sync, Redis, SQS)
- Prometheus metrics support
- Webhook support with batching
- SSL/TLS support
- CORS configuration
- Docker deployment support
- Multi-platform Docker images (amd64, arm64)
- Comprehensive documentation (English & Turkish)
- Example configurations and deployment guides

### Security
- GPL-3.0 License

[Unreleased]: https://github.com/ferdiunal/soketi.rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ferdiunal/soketi.rs/releases/tag/v0.1.0
