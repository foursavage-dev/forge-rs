# Production Deployment Guide

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

This guide provides comprehensive instructions for deploying Forge-rs in production environments.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Deployment Strategies](#deployment-strategies)
- [Monitoring and Observability](#monitoring-and-observability)
- [Security Considerations](#security-considerations)
- [Performance Tuning](#performance-tuning)
- [Troubleshooting](#troubleshooting)
- [Maintenance](#maintenance)

## Prerequisites

### System Requirements

- **Operating System**: Linux (Ubuntu 20.04+, Debian 11+, RHEL 8+), macOS 12+, Windows 10+
- **Rust**: 1.88 or later (MSRV 1.88)
- **Memory**: Minimum 4GB RAM, 8GB+ recommended for large projects
- **Disk Space**: 10GB+ for cache storage (configurable)
- **CPU**: Multi-core CPU recommended for parallel builds

### Required Dependencies

#### Linux
```bash
# Build tools
sudo apt-get update
sudo apt-get install -y build-essential cmake git curl

# Language toolchains (optional, based on your project needs)
sudo apt-get install -y gcc g++ python3 nodejs npm golang openjdk-17-jdk
```

#### macOS
```bash
# Build tools
xcode-select --install

# Language toolchains (optional)
brew install python3 node go openjdk@17
```

#### Windows
```powershell
# Build tools
# Install Visual Studio Build Tools or Visual Studio Community

# Language toolchains (optional)
choco install python nodejs golang openjdk17
```

## Installation

### Method 1: Cargo Install

```bash
cargo install forge-cli --release
```

### Method 2: Build from Source

```bash
git clone https://github.com/foursavage-dev/forge-rs.git
cd forge-rs
cargo build --release --workspace
cargo install --path crates/forge-cli
```

### Method 3: Binary Download

Download the latest release from [GitHub Releases](https://github.com/foursavage-dev/forge-rs/releases).

```bash
# Linux
wget https://github.com/foursavage-dev/forge-rs/releases/latest/download/forge-linux-x86_64
chmod +x forge-linux-x86_64
sudo mv forge-linux-x86_64 /usr/local/bin/forge

# macOS
wget https://github.com/foursavage-dev/forge-rs/releases/latest/download/forge-darwin-x86_64
chmod +x forge-darwin-x86_64
sudo mv forge-darwin-x86_64 /usr/local/bin/forge

# Windows
# Download forge-windows-x86_64.exe and add to PATH
```

## Configuration

### Forge Configuration File

Create a `forge.toml` in your project root:

```toml
[general]
# Cache directory
cache_dir = "~/.forge/cache"
# Maximum cache size (e.g., "10GB", "500MB")
max_cache_size = "10GB"
# Number of parallel jobs
parallel_jobs = 4
# Enable verbose logging
verbose = false

[cache]
# Enable local caching
enabled = true
# Remote cache URL (optional)
remote_url = "https://cache.example.com"
# Cache TTL in seconds
ttl = 86400

[build]
# Enable incremental builds
incremental = true
# Target directory
target_dir = "target"
# Build mode: debug, release, or both
mode = "release"

[security]
# Security level: strict, moderate, or permissive
level = "strict"
# Enable sandboxing
sandbox = true
# Allowed executable paths
allowed_executables = ["/usr/bin/cargo", "/usr/bin/rustc"]

[ci]
# CI platform: github, gitlab, circleci, or bitbucket
platform = "github"
# Enable CI caching
cache_enabled = true
# Remote cache URL for CI
remote_cache_url = "https://cache.example.com"
```

### Environment Variables

```bash
# Override configuration with environment variables
export FORGE_CACHE_DIR="/custom/cache/dir"
export FORGE_MAX_CACHE_SIZE="20GB"
export FORGE_PARALLEL_JOBS="8"
export FORGE_VERBOSE="true"
export FORGE_SECURITY_LEVEL="strict"
```

## Deployment Strategies

### Strategy 1: Development Environment

For local development, use default settings with local caching:

```bash
forge build
forge test
forge check
```

### Strategy 2: CI/CD Pipeline

Integrate Forge into your CI pipeline:

#### GitHub Actions
```yaml
name: Build with Forge

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Forge
        run: cargo install forge-cli --release
      - name: Build with Forge
        run: forge build --no-cache
      - name: Test with Forge
        run: forge test --no-cache
```

#### GitLab CI
```yaml
build:
  image: rust:latest
  script:
    - cargo install forge-cli --release
    - forge build --no-cache
    - forge test --no-cache
```

### Strategy 3: Distributed Build System

For large-scale deployments, use distributed builds with remote caching:

```toml
[cache]
enabled = true
remote_url = "https://cache.example.com"
remote_cache_enabled = true

[build]
distributed = true
worker_pool_size = 10
```

### Strategy 4: Kubernetes Deployment

Deploy Forge as a Kubernetes deployment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: forge-builder
spec:
  replicas: 3
  selector:
    matchLabels:
      app: forge-builder
  template:
    metadata:
      labels:
        app: forge-builder
    spec:
      containers:
      - name: forge
        image: forge:latest
        command: ["forge", "build"]
        volumeMounts:
        - name: cache
          mountPath: /cache
      volumes:
      - name: cache
        persistentVolumeClaim:
          claimName: forge-cache-pvc
```

## Monitoring and Observability

### Health Checks

Forge provides built-in health checks:

```bash
forge health
```

Health check output:
```json
{
  "status": "healthy",
  "components": [
    {
      "name": "cache",
      "status": "healthy",
      "message": "Cache operational"
    },
    {
      "name": "executor",
      "status": "healthy",
      "message": "Executor ready"
    }
  ]
}
```

### Metrics Collection

Forge collects performance metrics:

```bash
forge metrics
```

Metrics include:
- Cache hit/miss rates
- Build duration
- Memory usage
- CPU utilization
- Task completion rates

### Logging

Configure logging levels:

```toml
[general]
# Log level: trace, debug, info, warn, error
log_level = "info"
# Log file path
log_file = "/var/log/forge/forge.log"
```

### Diagnostic Reports

Generate diagnostic reports:

```bash
forge diagnostics
```

## Security Considerations

### Security Levels

Configure security level based on your needs:

```toml
[security]
# strict: Maximum security, validates all inputs
# moderate: Balanced security and performance
# permissive: Minimal security checks
level = "strict"
```

### Sandbox Configuration

Enable sandboxing for untrusted builds:

```toml
[security]
sandbox = true
sandbox_type = "namespace"  # or "chroot"
allowed_network = false
allowed_filesystem = ["/safe/path"]
```

### Credential Management

Use Forge's secrets management:

```bash
forge secret set API_KEY "your-api-key"
forge secret get API_KEY
```

### Audit Logging

Enable audit logging for compliance:

```toml
[security]
audit_log = true
audit_log_path = "/var/log/forge/audit.log"
```

## Performance Tuning

### Cache Optimization

Optimize cache settings for your workload:

```toml
[cache]
# Increase cache size for better hit rates
max_cache_size = "50GB"
# Use SSD cache for faster access
cache_type = "ssd"
# Enable compression
compression = true
```

### Parallel Execution

Adjust parallel job count based on available CPU cores:

```toml
[general]
parallel_jobs = 8  # Set to number of CPU cores
```

### Memory Management

Configure memory limits:

```toml
[general]
max_memory = "8GB"
memory_limit_per_job = "2GB"
```

### Network Optimization

For remote cache configurations:

```toml
[cache]
remote_url = "https://cache.example.com"
remote_timeout = 30  # seconds
remote_concurrent_requests = 10
```

## Troubleshooting

### Common Issues

#### Issue: Build fails with cache error

**Solution**:
```bash
# Clear cache
forge cache clear

# Verify cache integrity
forge cache verify

# Rebuild without cache
forge build --no-cache
```

#### Issue: Out of memory errors

**Solution**:
```toml
[general]
parallel_jobs = 2  # Reduce parallel jobs
max_memory = "4GB"  # Increase memory limit
```

#### Issue: Slow builds

**Solution**:
```bash
# Check cache hit rate
forge cache stats

# Enable remote cache
# Update forge.toml with remote_url

# Increase parallel jobs
# Update forge.toml with parallel_jobs
```

#### Issue: Network timeout with remote cache

**Solution**:
```toml
[cache]
remote_timeout = 60  # Increase timeout
remote_concurrent_requests = 5  # Reduce concurrent requests
```

### Debug Mode

Enable debug logging for troubleshooting:

```bash
forge build --verbose --log-level debug
```

### Diagnostic Collection

Collect diagnostic information:

```bash
forge diagnostics --output forge-diagnostics.json
```

## Maintenance

### Cache Maintenance

Regular cache maintenance:

```bash
# Prune old cache entries
forge cache prune --older-than 30d

# Compress cache
forge cache compress

# Verify cache integrity
forge cache verify
```

### Log Rotation

Configure log rotation:

```toml
[general]
log_file = "/var/log/forge/forge.log"
log_max_size = "100MB"
log_max_files = 10
```

### Updates

Update Forge to the latest version:

```bash
cargo install forge-cli --force
```

### Backup and Recovery

Backup Forge configuration and cache:

```bash
# Backup configuration
cp forge.toml forge.toml.backup

# Backup cache
tar -czf forge-cache-backup.tar.gz ~/.forge/cache

# Restore cache
tar -xzf forge-cache-backup.tar.gz -C ~/
```

## Support

For issues and questions:
- GitHub Issues: https://github.com/foursavage-dev/forge-rs/issues
- Documentation: https://github.com/foursavage-dev/forge-rs/blob/main/README.md
- Discord: [Join our Discord server]

## License

MIT License - See LICENSE file for details.