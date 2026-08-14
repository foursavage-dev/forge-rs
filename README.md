# Forge 🦀

[![CI](https://github.com/requla11/forge-rs/actions/workflows/dogfood.yml/badge.svg)](https://github.com/requla11/forge-rs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![AI Assistant](https://img.shields.io/badge/AI%20Bot-Gemini%203.7%20Flash-brightgreen.svg)](https://github.com/requla11/forge-rs/actions)
[![Language Backends](https://img.shields.io/badge/backends-10%2B-ff69b4.svg)](https://github.com/requla11/forge-rs)
[![Cache Engine](https://img.shields.io/badge/cache-CAS%20Artifact-blue.svg)](https://github.com/requla11/forge-rs)
[![CI Generator](https://img.shields.io/badge/CI%20Generator-Auto%20Generate-green.svg)](https://github.com/requla11/forge-rs)
[![Stars](https://img.shields.io/github/stars/requla11/forge-rs?style=social)](https://github.com/requla11/forge-rs/stargazers)
[![Forks](https://img.shields.io/github/forks/requla11/forge-rs?style=social)](https://github.com/requla11/forge-rs/network/members)

> A blazing fast, polyglot, cache-first build orchestration system built in Rust.
> Forge orchestrates your toolchains into an incremental, cache-aware, and distributed build graph.

Forge is **not** a compiler, a package registry, or a language package manager replacement. It orchestrates existing toolchains (`rustc`, `cargo`, `clang`, `gcc`, `go`, `node`, `dotnet`, `javac`, `swiftc`, `zig`, `docker`, ...) behind a dependency graph, parallel scheduler, executor, CAS artifact cache, and distributed cluster.

---

## ⚡ Quick Installation

### One-Line Install (Recommended)

**Linux & macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/requla11/forge-rs/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/requla11/forge-rs/main/install.ps1 | iex
```

### From Source
```bash
cargo install --path crates/forge-cli
```

### Cargo Install
```bash
cargo install forge-cli --git https://github.com/requla11/forge-rs
```

---

## 🚀 Key Highlights

- ⚡ **Instant Cache-First Rebuilds:** Fingerprints file content (Blake3), lockfiles, toolchain versions, and dependency cones for instant `0.00s` incremental builds.
- 🌐 **Polyglot Monorepo Support:** Out-of-the-box backends for 10+ languages and containers without complex Starlark/BUILD configurations.
- 📦 **CAS Artifact Cache:** Content-Addressable Storage for local and remote caching of exact build outputs and binaries.
- 🔄 **Level Partitioning & Batching:** Groups independent packages per build level into single toolchain calls, eliminating process spawn overhead.
- 🤖 **AI Assistant Bot (`requla11-bot[bot]`):** Powered by Gemini 3.7 Flash for automatic PR benchmarking, affected crate reviews, and issue triage.
- 📊 **Interactive TUI & Web Dashboard:** Live Terminal UI with Ratatui and an interactive Web Flamegraph visualizer for bottleneck discovery.
- ☸️ **Distributed Cluster Execution:** Remote worker discovery, tiered L1/L2 cache daemons, and failover balancing.
- 🛠️ **CI/CD Generator:** Automatically generate optimized GitHub Actions and GitLab CI workflows with `forge ci init`.

---

## 🌐 Supported Language Backends

Forge supports **10+ language backends** out of the box with automatic project detection:

|| Backend | Primary Manifest / Config | Compilers / Toolchains | Features |
|| :--- | :--- | :--- | :--- |
|| **Rust** | `Cargo.toml` | `cargo`, `rustc` | Level-batched build/test/check, workspace support |
|| **C / C++** | `forge.cc.json` | `gcc`, `clang`, `msvc` | Header dependency tracking, parallel compilation |
|| **Go** | `go.mod`, `forge.go.json` | `go build`, `go test`, `go vet` | Module-aware, test discovery |
|| **TypeScript / JS** | `package.json`, `forge.ts.json` | `npm`, `pnpm`, `yarn`, `bun` | Monorepo support, TypeScript compilation |
|| **Docker / OCI** | `Dockerfile`, `forge.docker.json` | `docker buildx`, `podman` | Multi-stage builds, layer caching |
|| **Python** | `pyproject.toml`, `forge.py.json` | `uv`, `poetry`, `pip`, `pytest` | Virtual environment, dependency management |
|| **Java / Kotlin** | `pom.xml`, `build.gradle` | `javac`, `mvn`, `gradle`, `kotlinc` | Maven & Gradle support, Kotlin compilation |
|| **.NET / C# / F#** | `*.csproj`, `*.sln` | `dotnet build`, `dotnet test` | Multi-target framework support |
|| **Swift / Objective-C** | `Package.swift`, `*.xcodeproj` | `swift build`, `swift test`, `clang` | iOS/macOS/tvOS/watchOS targets |
|| **Dart / Flutter** | `pubspec.yaml` | `dart compile`, `flutter build` | Web, mobile, desktop targets |
|| **Zig** | `build.zig` | `zig build`, `zig test` | Cross-compilation, native targets |
|| **Custom Plugins** | `forge.plugin.json` | Any executable CLI tool | Extensible rule system |

---

## 🖥️ Command Line Interface

|| Command | Description |
|| :--- | :--- |
|| `forge build [-j N] [--no-cache] [--sandbox]` | Build the workspace or target project |
|| `forge check` | Run type checking and linters across all backends |
|| `forge test` | Run test suites across the dependency graph |
|| `forge run [-p PKG] [--bin BIN]` | Build dependencies and run target binary |
|| `forge graph [--format tree\|json\|dot]` | Render visual dependency DAG graph |
|| `forge watch [--mode build\|test]` | Auto-rebuild on file changes with debounce |
|| `forge affected [--since REV]` | List packages affected by git commit/branch |
|| `forge doctor` | Inspect environment and toolchain readiness |
|| `forge ci init [--type github\|gitlab]` | Generate optimized CI matrix workflows |
|| `forge dashboard [--port 8080]` | Start interactive Web UI & Flamegraph |
|| `forge cache stats / prune` | Inspect and clean local CAS fingerprint cache |
|| `forge cache-server [--listen ADDR]` | Start remote artifact cache daemon |
|| `forge worker [--listen ADDR]` | Start distributed build execution worker |
|| `forge clean` | Clean workspace build artifacts |

---

## 📚 Advanced Features

### CAS Artifact Cache
Content-Addressable Storage for efficient artifact caching:
- **Local storage**: Compressed artifact storage with BLAKE3 hashing
- **Remote support**: AWS S3, GCS, MinIO integration
- **Compression**: Zstandard compression with configurable levels
- **CLI commands**: `forge cache cas upload/download/list/stats/cleanup`

### CI/CD Generator
Automatically generate optimized CI workflows:
- **GitHub Actions**: `forge ci init --platform github`
- **GitLab CI**: `forge ci init --platform gitlab`
- **Matrix generation**: Parallel job scheduling based on build graph
- **Cache integration**: Automatic cache configuration
- **Affected builds**: PR-optimized CI for changed packages only

### Web Dashboard & Flamegraph
Interactive performance visualization:
- **Build DAG visualization**: Real-time build graph display
- **Flamegraph analysis**: Task timing and bottleneck identification
- **Metrics tracking**: Cache hit rates, build times, success rates
- **Auto-refresh**: Live updates during builds

---

## 🏗️ Workspace Architecture

```text
forge/
├── crates/
│   ├── forge-core/            # Workspace discovery & package model
│   ├── forge-graph/           # Build graph: nodes, edges, states, topo order
│   ├── forge-executor/        # Task model, CommandSpec, process execution
│   ├── forge-scheduler/       # Parallel ready-queue scheduler
│   ├── forge-cache/           # Fingerprint store & executor cache wrapper
│   ├── forge-cas/             # Content-Addressable Storage (CAS) engine
│   ├── forge-remote-cache/    # Remote cache client & tiered L1/L2 composite
│   ├── forge-worker/          # Distributed execution worker & clustering
│   ├── forge-sandbox/         # Hermetic environment isolation
│   ├── forge-ci-generator/    # GitHub Actions & GitLab CI pipeline generator
│   ├── forge-backend-rust/    # Cargo metadata -> task graph + fingerprints
│   ├── forge-backend-cc/      # C/C++ backend (gcc/clang/msvc)
│   ├── forge-backend-go/      # Go backend (go.mod)
│   ├── forge-backend-ts/      # TypeScript/JS backend (package.json)
│   ├── forge-backend-docker/  # Docker & container packaging backend
│   ├── forge-backend-py/      # Python backend (pyproject.toml)
│   ├── forge-backend-java/    # Java backend (Maven/Gradle)
│   ├── forge-backend-dotnet/  # .NET / C# backend (csproj/sln)
│   ├── forge-backend-swift/   # Swift Package Manager backend
│   ├── forge-backend-dart/    # Dart / Flutter backend
│   ├── forge-backend-zig/     # Zig backend (build.zig)
│   ├── forge-plugin/          # Custom rule plugin backend
│   └── forge-cli/             # The `forge` binary & terminal UI
├── examples/
│   └── polyglot-demo/         # Sample monorepo with Rust + Go + TS + Docker
├── install.sh                 # One-line installer for Linux/macOS
├── install.ps1                # One-line installer for Windows
└── .github/workflows/         # CI/CD, Dogfooding, and AI Bot Workflows
```

---

## 🎯 Quick Start Examples

### Build a Rust Project
```bash
cd your-rust-project
forge build
```

### Build a Polyglot Monorepo
```bash
# Clone the example monorepo
git clone https://github.com/requla11/forge-rs.git
cd forge-rs/examples/polyglot-demo

# Build all services (Rust + Go + TypeScript + Docker)
forge build

# View the build graph
forge graph

# Run tests
forge test
```

### Generate CI Configuration
```bash
# Generate GitHub Actions workflow
forge ci init --platform github

# Generate GitLab CI pipeline
forge ci init --platform gitlab
```

### Use CAS Artifact Cache
```bash
# Upload build artifacts
forge cache cas upload target/release/my_binary

# List cached artifacts
forge cache cas list

# Download by hash
forge cache cas download <hash> --output my_binary
```

---

## 🦀 Forge Builds Forge

This repository dogfoods itself. CI compiles Forge from source and then uses Forge to build, check, test, and profile the entire workspace:

```bash
cargo build --release            # Bootstrap forge
./target/release/forge build     # Forge builds all 20 crates in parallel
./target/release/forge test      # Run all workspace test suites
./target/release/forge graph --format dot
```

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.
