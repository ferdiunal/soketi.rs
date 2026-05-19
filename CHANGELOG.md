# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.7](https://github.com/ferdiunal/soketi.rs/compare/v1.2.6...v1.2.7) (2026-05-19)

### 🐛 Bug Fixes

* bind api signatures to request bodies ([172fb47](https://github.com/ferdiunal/soketi.rs/commit/172fb47f91ebec0de4005b23453572944bca927d))
* clean websocket state on all close paths ([04472e2](https://github.com/ferdiunal/soketi.rs/commit/04472e27a968286b4abe488d124a082ebd8e6128))
* enforce configured api body limits ([31d98fa](https://github.com/ferdiunal/soketi.rs/commit/31d98fa82fe21724b159261e2d93bb846d4b042d))
* enforce websocket app policies ([c29b3b5](https://github.com/ferdiunal/soketi.rs/commit/c29b3b53d77ce2f018a6dd73c6ea4f329ad26319))

## [1.2.6](https://github.com/ferdiunal/soketi.rs/compare/v1.2.5...v1.2.6) (2026-05-05)

### 🐛 Bug Fixes

* **config:** add struct-level serde(default) to all config structs ([44166a9](https://github.com/ferdiunal/soketi.rs/commit/44166a944b26f72e2df00c82f01c0cb2f2dc28a9))

## [1.2.5](https://github.com/ferdiunal/soketi.rs/compare/v1.2.4...v1.2.5) (2026-05-05)

### 🐛 Bug Fixes

* **config:** add serde(default) to all optional sub-config fields ([0f4382d](https://github.com/ferdiunal/soketi.rs/commit/0f4382d207a5fb9f6e08bd9136c246c8ee531041))

## [1.2.4](https://github.com/ferdiunal/soketi.rs/compare/v1.2.3...v1.2.4) (2026-05-05)

### 🐛 Bug Fixes

* **docker:** add static wget from busybox for shell-free healthcheck ([f57aaba](https://github.com/ferdiunal/soketi.rs/commit/f57aabac8b8c9bb2fb89a9ac4bd886542d5867ef))

## [1.2.3](https://github.com/ferdiunal/soketi.rs/compare/v1.2.2...v1.2.3) (2026-05-05)

### ⚡ Performance Improvements

* **docker:** use pre-built musl binaries instead of compiling in Docker ([52c2bbe](https://github.com/ferdiunal/soketi.rs/commit/52c2bbe34c9547620ecb45b12a6f37a7447935d2))

## [1.2.2](https://github.com/ferdiunal/soketi.rs/compare/v1.2.1...v1.2.2) (2026-05-05)

### 🐛 Bug Fixes

* **ci:** replace cross with cargo-zigbuild for Linux musl builds ([453316f](https://github.com/ferdiunal/soketi.rs/commit/453316f2d18b0761204f81abca2fba1fbac65769))

## [1.2.1](https://github.com/ferdiunal/soketi.rs/compare/v1.2.0...v1.2.1) (2026-05-05)

### 🐛 Bug Fixes

* **ci:** use DOCKER_HUB_TOKEN secret name for Docker Hub login ([a0f58e9](https://github.com/ferdiunal/soketi.rs/commit/a0f58e904d5b38782c8e8ffb95e5f2b1aca4fe0b))

## [1.2.0](https://github.com/ferdiunal/soketi.rs/compare/v1.1.1...v1.2.0) (2026-05-05)

### ✨ Features

* **docker:** switch to distroless runtime and add multi-arch binary releases ([f52fa85](https://github.com/ferdiunal/soketi.rs/commit/f52fa851d061c9f30add66861ceebe30d6543cf2))

## [1.1.1](https://github.com/ferdiunal/soketi.rs/compare/v1.1.0...v1.1.1) (2026-03-14)

### ♻️ Code Refactoring

* **ci:** enhance Docker release workflow for multi-architecture support ([6a3b5e1](https://github.com/ferdiunal/soketi.rs/commit/6a3b5e1ae979e26ee0d88fec785cc0b5a1a21fa0))

## [1.1.0](https://github.com/ferdiunal/soketi.rs/compare/v1.0.2...v1.1.0) (2026-02-22)

### ✨ Features

* **ci:** add Docker release workflow and update Dockerfile ([796eb9d](https://github.com/ferdiunal/soketi.rs/commit/796eb9d1b42dac19189240278596592358236c10))

### 📚 Documentation

* **_Sidebar.md:** add English language section header ([620d0ec](https://github.com/ferdiunal/soketi.rs/commit/620d0ec8728935396d5f04fa15456d473d495914))
* update documentation structure and release configuration ([cdd5799](https://github.com/ferdiunal/soketi.rs/commit/cdd5799e669abd9b4c59780c8e828997e48b7ca2))

## [1.0.2](https://github.com/ferdiunal/soketi.rs/compare/v1.0.1...v1.0.2) (2026-01-25)

### 📚 Documentation

* **HOME.md:** update GitHub repository URLs to use correct domain ([cbc57a9](https://github.com/ferdiunal/soketi.rs/commit/cbc57a90e06733112b514c3746bbd7b594755dad))

## [1.0.1](https://github.com/ferdiunal/soketi.rs/compare/v1.0.0...v1.0.1) (2026-01-25)

### 📚 Documentation

* update documentation, add environment examples, and refactor CI/CD ([677b176](https://github.com/ferdiunal/soketi.rs/commit/677b176a8cbfd081f4f7e0bd0362fdf5efa93d65))

## 1.0.0 (2026-01-25)

### 🐛 Bug Fixes

* Update Dockerfile path and license to GPL-3.0 ([00bca6c](https://github.com/ferdiunal/soketi.rs/commit/00bca6c968de67fe22aafd4d90ded2a7de7ea283))

### 📚 Documentation

* add Docker Hub README and update license information ([271eafd](https://github.com/ferdiunal/soketi.rs/commit/271eafd8bca175f5aae77cf05f1d04c8fe7ba8d0))
* add release and versioning documentation ([9609c8b](https://github.com/ferdiunal/soketi.rs/commit/9609c8bd92cd452928d7f2361d766478341f0f88))

### 👷 CI/CD

* **.github/workflows:** update Docker username in publish workflow ([07e0f1d](https://github.com/ferdiunal/soketi.rs/commit/07e0f1d62b4ea1fef2b4e519c6bfc7063aefc47a))

## [Unreleased]

### Security

- Enforced WebSocket app policy checks on the deployed route (`c29b3b5`).
- Bound signed HTTP API requests to their request bodies (`172fb47`).
- Cleaned WebSocket adapter state on all close paths (`04472e2`).
- Applied configured HTTP API body limits and live validation config (`31d98fa`).

### Performance

- Reduced local adapter send overhead (`d64b3ed`).
- Reduced client event hot-path overhead (`2390870`).
- Stabilized the local end-to-end latency benchmark (`c3bd38a`).

### Documentation

- Added bilingual security policy and recent security hardening summary.
- Added bilingual Cloudflare Containers deployment documentation.

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
