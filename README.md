# Forge 🦀

[![CI](https://github.com/requla11/forge-rs/actions/workflows/dogfood.yml/badge.svg)](https://github.com/requla11/forge-rs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![AI Assistant](https://img.shields.io/badge/AI%20Bot-Gemini%203.7%20Flash-brightgreen.svg)](https://github.com/requla11/forge-rs/actions)

> A blazing fast, polyglot, cache-first build orchestration system built in Rust.
> Forge orchestrates your toolchains into an incremental, cache-aware, and distributed build graph.

Forge is **not** a compiler, a package registry, or a language package manager replacement. It orchestrates existing toolchains (`rustc`, `cargo`, `clang`, `gcc`, `go`, `node`, `dotnet`, `javac`, `swiftc`, `zig`, `docker`, ...) behind a dependency graph, parallel scheduler, executor, CAS artifact cache, and distributed cluster.

---

## ⚡ Quick Installation

### Linux & macOS
```bash
curl -fsSL https://raw.githubusercontent.com/requla11/forge-rs/main/install.sh | bash
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/requla11/forge-rs/main/install.ps1 | iex
```

### From Source
```bash
cargo install --path crates/forge-cli
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

| Backend | Primary Manifest / Config | Compilers / Toolchains |
| :--- | :--- | :--- |
| **Rust** | `Cargo.toml` | `cargo`, `rustc` (Level-batched build/test/check) |
| **C / C++** | `forge.cc.json` | `gcc`, `clang`, `msvc` (with `.d` header depfiles) |
| **Go** | `go.mod`, `forge.go.json` | `go build`, `go test`, `go vet` |
| **TypeScript / JS** | `package.json`, `forge.ts.json` | `npm`, `pnpm`, `yarn`, `bun` |
| **Docker / OCI** | `Dockerfile`, `forge.docker.json` | `docker buildx`, `podman` |
| **Python** | `pyproject.toml`, `forge.py.json` | `uv`, `poetry`, `pip`, `pytest` |
| **Java** | `pom.xml`, `build.gradle` | `javac`, `mvn`, `gradle` |
| **.NET / C#** | `*.csproj`, `*.sln` | `dotnet build`, `dotnet test` |
| **Swift** | `Package.swift` | `swift build`, `swift test` |
| **Dart / Flutter** | `pubspec.yaml` | `dart compile`, `flutter build` |
| **Zig** | `build.zig` | `zig build` |
| **Custom Plugins** | `forge.plugin.json` | Any executable CLI tool / custom rule |

---

## 🖥️ Command Line Interface

| Command | Description |
| :--- | :--- |
| `forge build [-j N] [--no-cache] [--sandbox]` | Build the workspace or target project |
| `forge check` | Run type checking and linters across all backends |
| `forge test` | Run test suites across the dependency graph |
| `forge run [-p PKG] [--bin BIN]` | Build dependencies and run target binary |
| `forge graph [--format tree\|json\|dot]` | Render visual dependency DAG graph |
| `forge watch [--mode build\|test]` | Auto-rebuild on file changes with debounce |
| `forge affected [--since REV]` | List packages affected by git commit/branch |
| `forge doctor` | Inspect environment and toolchain readiness |
| `forge ci init [--type github\|gitlab]` | Generate optimized CI matrix workflows |
| `forge dashboard [--port 8080]` | Start interactive Web UI & Flamegraph |
| `forge cache stats / prune` | Inspect and clean local CAS fingerprint cache |
| `forge cache-server [--listen ADDR]` | Start remote artifact cache daemon |
| `forge worker [--listen ADDR]` | Start distributed build execution worker |
| `forge clean` | Clean workspace build artifacts |

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
│   ├── forge-dashboard/       # Web UI & Flamegraph performance visualizer
│   ├── forge-plugin/          # Custom rule plugin backend
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
│   └── forge-cli/             # The `forge` binary & terminal UI
└── .github/workflows/         # CI/CD, Dogfooding, and AI Bot Workflows
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

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.