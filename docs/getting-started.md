# Getting Started with Fish

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

This guide will help you get started with Fish, a fast, cache-first build orchestration system.

## Installation

### One-Line Install (Recommended)

**Linux & macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/requla11/fish/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/requla11/fish/main/install.ps1 | iex
```

### From Source

```bash
# Clone the repository
git clone https://github.com/requla11/fish.git
cd fish

# Build and install
cargo install --path crates/fish-cli
```

### Cargo Install

```bash
cargo install fish-cli --git https://github.com/requla11/fish
```

## Quick Start

### Building a Rust Project

```bash
cd your-rust-project
fish build
```

### Building a Polyglot Monorepo

```bash
# Clone the example monorepo
git clone https://github.com/requla11/fish.git
cd fish/examples/polyglot-demo

# Build all services
fish build

# View the build graph
fish graph

# Run tests
fish test
```

## Basic Commands

### Build Commands

```bash
# Build the entire workspace
fish build

# Build specific package
fish build -p my-package

# Build with 8 parallel jobs
fish build -j 8

# Build without cache
fish build --no-cache

# Build with sandbox
fish build --sandbox

# Build with detailed dirty rebuild explanation
fish build --explain

# Profile-Guided Optimization (PGO) workflow
fish build --pgo-generate
# ... run your benchmarks/workload ...
fish build --pgo-use
```

### Graph & Query Commands

```bash
# Query transitive dependencies (Bazel-style)
Fish query "deps(//fish-cli)"

# Query reverse dependencies
Fish query "rdeps(//fish-graph)"

# Find all paths between two modules
Fish query "allpaths(//fish-cli, //fish-core)"

# Filter dependencies by regex
Fish query "filter('backend', deps(//fish-cli))"

# Visual graph rendering
fish graph --format tree
fish graph --format dot
```

### Build Daemon Commands

```bash
# Start background build daemon for sub-millisecond warm builds
Fish daemon start

# Check daemon status
Fish daemon status

# Stop background daemon
Fish daemon stop
```

### Test Commands

```bash
# Run all tests
fish test

# Test specific package
fish test -p my-package

# Test with cache disabled
fish test --no-cache
```

### Cache Commands

```bash
# View cache statistics
Fish cache stats

# Clean cache
Fish cache prune

# Start cache server
Fish cache-server --listen 0.0.0.0:8080
```

### Distributed Build Commands

```bash
# Start a worker
Fish worker --listen 0.0.0.0:9000

# Build with distributed workers
fish build --workers worker1:9000,worker2:9000
```

### CI/CD Commands

```bash
# Generate GitHub Actions workflow
fish ci init --platform github

# Generate GitLab CI pipeline
fish ci init --platform gitlab

# Generate CircleCI config
fish ci init --platform circleci

# Generate Bitbucket Pipelines
fish ci init --platform bitbucket

# Generate all platform configs
fish ci init --platform all
```

### Plugin Commands

```bash
# List available plugins
fish plugin list

# Execute a plugin command
fish plugin execute my-plugin build

# Install a plugin
fish plugin install ./my-plugin
```

## Configuration

### Workspace Configuration (`fish.toml`)

Fish automatically detects project types based on manifest files. For custom workspace execution, caching, and pipeline configuration, create a `fish.toml` in your project root:

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
dir = "~/.Fish/cache"
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

Fish includes a built-in real-time interactive DAG visualizer and telemetry dashboard with 5-language localization (English, Vietnamese, Simplified Chinese, Traditional Chinese, Japanese):

```bash
# Launch web dashboard on port 3000 and open in default browser
fish ui --port 3000 --open

# Check JSON graph data
curl http://localhost:3000/api/graph

# Check hardware and CAS stats
curl http://localhost:3000/api/stats
```

---

## Troubleshooting

### Build Fails

If a build fails:

1. Check the error message or run `fish build --explain` to diagnose rebuild reasons.
2. Run with debug logging: `RUST_LOG=debug fish build`
3. Verify toolchain readiness: `fish doctor`
4. Try clearing cache: `Fish cache prune`

### Cache Issues

If cache doesn't work:

1. Check cache stats: `Fish cache stats`
2. Verify cache directory is writable: `~/.Fish/cache`
3. Clear and rebuild cache: `Fish cache prune && fish build`

### Worker Connection Issues

If workers can't connect:

1. Check network connectivity
2. Verify worker is running: `Fish worker --listen 0.0.0.0:9000`
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
- [GitHub Issues](https://github.com/requla11/fish/issues)
- [Email](foursavage@proton.me)
