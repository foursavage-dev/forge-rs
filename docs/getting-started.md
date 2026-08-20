# Getting Started with Forge

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../TRANSLATION.md).

This guide will help you get started with Forge, a fast, cache-first build orchestration system.

## Installation

### One-Line Install (Recommended)

**Linux & macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/foursavage-dev/forge-rs/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/foursavage-dev/forge-rs/main/install.ps1 | iex
```

### From Source

```bash
# Clone the repository
git clone https://github.com/foursavage-dev/forge-rs.git
cd forge-rs

# Build and install
cargo install --path crates/forge-cli
```

### Cargo Install

```bash
cargo install forge-cli --git https://github.com/foursavage-dev/forge-rs
```

## Quick Start

### Building a Rust Project

```bash
cd your-rust-project
forge build
```

### Building a Polyglot Monorepo

```bash
# Clone the example monorepo
git clone https://github.com/foursavage-dev/forge-rs.git
cd forge-rs/examples/polyglot-demo

# Build all services
forge build

# View the build graph
forge graph

# Run tests
forge test
```

## Basic Commands

### Build Commands

```bash
# Build the entire workspace
forge build

# Build specific package
forge build -p my-package

# Build with 8 parallel jobs
forge build -j 8

# Build without cache
forge build --no-cache

# Build with sandbox
forge build --sandbox

# Build with detailed dirty rebuild explanation
forge build --explain

# Profile-Guided Optimization (PGO) workflow
forge build --pgo-generate
# ... run your benchmarks/workload ...
forge build --pgo-use
```

### Graph & Query Commands

```bash
# Query transitive dependencies (Bazel-style)
forge query "deps(//forge-cli)"

# Query reverse dependencies
forge query "rdeps(//forge-graph)"

# Find all paths between two modules
forge query "allpaths(//forge-cli, //forge-core)"

# Filter dependencies by regex
forge query "filter('backend', deps(//forge-cli))"

# Visual graph rendering
forge graph --format tree
forge graph --format dot
```

### Build Daemon Commands

```bash
# Start background build daemon for sub-millisecond warm builds
forge daemon start

# Check daemon status
forge daemon status

# Stop background daemon
forge daemon stop
```

### Test Commands

```bash
# Run all tests
forge test

# Test specific package
forge test -p my-package

# Test with cache disabled
forge test --no-cache
```

### Cache Commands

```bash
# View cache statistics
forge cache stats

# Clean cache
forge cache prune

# Start cache server
forge cache-server --listen 0.0.0.0:8080
```

### Distributed Build Commands

```bash
# Start a worker
forge worker --listen 0.0.0.0:9000

# Build with distributed workers
forge build --workers worker1:9000,worker2:9000
```

### CI/CD Commands

```bash
# Generate GitHub Actions workflow
forge ci init --platform github

# Generate GitLab CI pipeline
forge ci init --platform gitlab

# Generate CircleCI config
forge ci init --platform circleci

# Generate Bitbucket Pipelines
forge ci init --platform bitbucket

# Generate all platform configs
forge ci init --platform all
```

### Plugin Commands

```bash
# List available plugins
forge plugin list

# Execute a plugin command
forge plugin execute my-plugin build

# Install a plugin
forge plugin install ./my-plugin
```

## Configuration

### Workspace Configuration (`forge.toml`)

Forge automatically detects project types based on manifest files. For custom workspace execution, caching, and pipeline configuration, create a `forge.toml` in your project root:

```toml
[build]
backend = "auto"
jobs = 8
no_cache = false
sandbox = false
semantic = true
critical_path = true
ram_limit = 85

[cache]
dir = "~/.forge/cache"
reflink = true

[remote]
cache_url = "http://127.0.0.1:8080"
token = "secret-cache-token"

[daemon]
port = 9527

[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "Cargo.toml"]
outputs = ["target/release/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]
```

See [Configuration Guide](configuration.md) for full options.

---

## Interactive Telemetry & Web Dashboard

Forge includes a built-in real-time interactive DAG visualizer and telemetry dashboard with 5-language localization (English, Vietnamese, Simplified Chinese, Traditional Chinese, Japanese):

```bash
# Launch web dashboard on port 3000 and open in default browser
forge ui --port 3000 --open

# Check JSON graph data
curl http://localhost:3000/api/graph

# Check hardware and CAS stats
curl http://localhost:3000/api/stats
```

---

## Troubleshooting

### Build Fails

If a build fails:

1. Check the error message or run `forge build --explain` to diagnose rebuild reasons.
2. Run with debug logging: `RUST_LOG=debug forge build`
3. Verify toolchain readiness: `forge doctor`
4. Try clearing cache: `forge cache prune`

### Cache Issues

If cache doesn't work:

1. Check cache stats: `forge cache stats`
2. Verify cache directory is writable: `~/.forge/cache`
3. Clear and rebuild cache: `forge cache prune && forge build`

### Worker Connection Issues

If workers can't connect:

1. Check network connectivity
2. Verify worker is running: `forge worker --listen 0.0.0.0:9000`
3. Check firewall settings and authentication tokens
4. Review worker logs

## Next Steps

- Read the [Architecture Guide](architecture.md)
- Check the [Development Guide](../DEVELOPMENT.md)
- Explore [CLI Reference](cli-reference.md)
- Explore [Backend Documentation](backends/)

## Getting Help

- [Documentation](../README.md)
- [Support](../SUPPORT.md)
- [GitHub Issues](https://github.com/foursavage-dev/forge-rs/issues)
- [Email](foursavage@proton.me)
