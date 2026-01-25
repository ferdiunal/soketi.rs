# Installation Guide

> Comprehensive installation instructions for Soketi across different platforms and environments.

## Table of Contents

- [System Requirements](#system-requirements)
- [Docker Installation](#docker-installation)
- [Building from Source](#building-from-source)
- [Pre-built Binaries](#pre-built-binaries)
- [Platform-Specific Instructions](#platform-specific-instructions)
- [Verification](#verification)

## System Requirements

- **Operating System**: Linux, macOS, or Windows
- **Memory**: Minimum 512MB RAM (2GB+ recommended for production)
- **CPU**: 1+ cores (2+ cores recommended for production)
- **Network**: Open ports 6001 (WebSocket) and 9601 (Metrics)

For building from source:
- **Rust**: 1.70 or higher
- **Cargo**: Latest version

## Docker Installation

Docker is the recommended way to run Soketi for most use cases.

### Pull the Image

```bash
docker pull quay.io/soketi/soketi:latest-16-alpine
```

### Run with Default Configuration

```bash
docker run -p 6001:6001 -p 9601:9601 \
  -e SOKETI_DEFAULT_APP_ID=app-id \
  -e SOKETI_DEFAULT_APP_KEY=app-key \
  -e SOKETI_DEFAULT_APP_SECRET=app-secret \
  quay.io/soketi/soketi:latest-16-alpine
```

### Run with Custom Configuration

Create a `config.json` file and mount it:

```bash
docker run -p 6001:6001 -p 9601:9601 \
  -v $(pwd)/config.json:/app/config.json \
  quay.io/soketi/soketi:latest-16-alpine \
  --config /app/config.json
```

### Using Docker Compose

Create a `docker-compose.yml`:

```yaml
version: '3.8'

services:
  soketi:
    image: quay.io/soketi/soketi:latest-16-alpine
    ports:
      - "6001:6001"
      - "9601:9601"
    environment:
      - SOKETI_DEFAULT_APP_ID=app-id
      - SOKETI_DEFAULT_APP_KEY=app-key
      - SOKETI_DEFAULT_APP_SECRET=app-secret
    volumes:
      - ./config.json:/app/config.json
    command: --config /app/config.json
```

Start with:

```bash
docker-compose up -d
```

## Building from Source

### Prerequisites

Install Rust and Cargo:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/soketi/soketi.rs.git
cd soketi.rs

# Build in release mode
cargo build --release

# The binary will be at target/release/soketi
```

### Run the Binary

```bash
./target/release/soketi --config config.json
```

### Install System-wide (Optional)

```bash
sudo cp target/release/soketi /usr/local/bin/
soketi --version
```

## Pre-built Binaries

Download pre-built binaries from the [GitHub Releases page](https://github.com/soketi/soketi.rs/releases).

### Linux

```bash
# Download the latest release
wget https://github.com/soketi/soketi.rs/releases/latest/download/soketi-linux-x64

# Make it executable
chmod +x soketi-linux-x64

# Run it
./soketi-linux-x64 --config config.json
```

### macOS

```bash
# Download the latest release
curl -L https://github.com/soketi/soketi.rs/releases/latest/download/soketi-macos-x64 -o soketi

# Make it executable
chmod +x soketi

# Run it
./soketi --config config.json
```

### Windows

Download `soketi-windows-x64.exe` from the releases page and run it from Command Prompt or PowerShell:

```powershell
.\soketi-windows-x64.exe --config config.json
```

## Platform-Specific Instructions

### Ubuntu/Debian

```bash
# Install dependencies
sudo apt-get update
sudo apt-get install -y curl build-essential

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build Soketi
git clone https://github.com/soketi/soketi.rs.git
cd soketi.rs
cargo build --release
```

### CentOS/RHEL

```bash
# Install dependencies
sudo yum groupinstall -y "Development Tools"
sudo yum install -y curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build Soketi
git clone https://github.com/soketi/soketi.rs.git
cd soketi.rs
cargo build --release
```

### macOS

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build Soketi
git clone https://github.com/soketi/soketi.rs.git
cd soketi.rs
cargo build --release
```

## Verification

After installation, verify that Soketi is working:

### Check Version

```bash
soketi --version
```

### Start with Default Configuration

```bash
soketi
```

You should see output similar to:

```
[INFO] Starting Soketi server...
[INFO] WebSocket server listening on 0.0.0.0:6001
[INFO] Metrics server listening on 0.0.0.0:9601
```

### Test Connection

Use curl to check the health endpoint:

```bash
curl http://localhost:6001/
```

Expected response:

```json
{
  "version": "1.0.0",
  "status": "ok"
}
```

## Next Steps

- **[Configuration Guide](configuration.md)** - Configure Soketi for your needs
- **[Getting Started](getting-started.md)** - Quick start guide
- **[Deployment Guide](deployment/reverse-proxy.md)** - Deploy to production

## Related Resources

- [GitHub Repository](https://github.com/soketi/soketi.rs)
- [Docker Hub](https://quay.io/repository/soketi/soketi)
- [Troubleshooting Guide](troubleshooting.md)
