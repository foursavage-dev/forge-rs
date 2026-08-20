# Forge Architecture Guide

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../TRANSLATION.md).

This document provides a comprehensive technical overview of Forge's system architecture, core engine modules, and execution pipeline.

---

## System Overview

Forge is a high-performance, cache-first build orchestration system designed for polyglot monorepos and distributed development. Rather than replacing native compilers, Forge acts as an intelligent coordination layer across language toolchains, managing dependency DAGs, content-addressable caching (CAS), hermetic isolation, and parallel work-stealing execution.

```text
┌─────────────────────────────────────────────────────────────┐
│                    forge-cli / Web UI                       │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│       forge-core (Discovery, Toolchains, compile_commands)  │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│           forge-graph (DAG & Algebraic Query Engine)        │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│   forge-scheduler (Governor, Jobserver, Racing, Watcher)    │
└──────────────┬──────────────────────────────┬───────────────┘
               │                              │
┌──────────────▼──────────────┐┌──────────────▼──────────────┐
│ forge-executor & Middleware ││  forge-cache & forge-cas    │
└──────────────┬──────────────┘└─────────────────────────────┘
               │
┌──────────────▼──────────────────────────────────────────────┐
│      11+ Language Backends & Distributed Worker Network     │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Crates and Responsibilities

### 1. Workspace Discovery (`forge-core`)
- **Manifest Discovery**: Scans and parses `Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, `CMakeLists.txt`, `pom.xml`, `*.csproj`, `Package.swift`, `pubspec.yaml`, `build.zig`, `Dockerfile`.
- **Compilation Database**: Generates standard `compile_commands.json` for Clangd and IDEs (`CompilationDatabase`).
- **Hermetic Toolchains**: Manages and isolates toolchain paths and environments (`ToolchainRegistry`, `ToolchainSpec`).
- **Micro-Input Filtering**: Glob-based file filtering reducing cache invalidation churn (`MicroInputFilter`).

### 2. Build Graph (`forge-graph`)
- **Topological Task Graph**: Constructs a Directed Acyclic Graph (DAG) of build tasks with cycle detection.
- **Algebraic Graph Queries**: Evaluates Bazel-style graph expressions (`deps()`, `rdeps()`, `allpaths()`, `somepath()`, `filter()`).
- **Dynamic Node Expansion**: Generates sub-task graphs on the fly during execution (`DynamicGraphExpander`).

### 3. Execution & Fast Materialization (`forge-executor`)
- **Process Orchestration**: Non-blocking asynchronous task execution with timeouts and output stream capture.
- **Fast Extents Cloning**: Copy-on-Write (CoW) extents and hardlink materialization (`KernelCowCloner`).
- **Linker Dispatcher**: Auto-detects and synthesizes flags for `mold`, `lld`, `lld-link`, and `msvc` (`LinkerDispatcher`).
- **Compiler Response Files**: Generates `@forge_args.rsp` files when command arguments exceed operating system limits.

### 4. Scheduler & Resource Control (`forge-scheduler`)
- **Parallel Work-Stealing**: Lock-free task scheduling across available hardware cores.
- **Kernel Resource Governor**: Monitors memory pressure and dynamically throttles concurrency to prevent out-of-memory thrashing (`KernelResourceGovernor`).
- **Compiler Pipelining**: Coordinates multi-stage compilation to unblock downstream targets upon metadata readiness (`PipelinedCompilationCoordinator`).
- **GNU Jobserver Pool**: Global token pool coordinating thread allocation across nested compiler invocations (`JobserverPool`).
- **Dynamic Remote Racing**: Races local execution against distributed cluster workers (`DynamicRacingExecutor`).
- **Distributed Task Execution (DTE)**: Longest Processing Time (LPT) bin-packing for multi-agent CI balancing (`DteBinPacker`).
- **Real-Time Filesystem Watcher**: Background daemon monitoring file events and pre-warming cache graphs (`FsWatcherDaemon`).

### 5. Content-Addressable Storage (`forge-cache` & `forge-cas`)
- **Fingerprinting**: Blake3 content hashing over source files, environment variables, and compiler flags.
- **CAS Storage**: Deduplicated artifact storage with Zstandard compression.
- **Tiered Composite Caching**: L1 local in-memory/disk cache and L2 remote S3/HTTP cache integration.

### 6. User Interface & Telemetry (`forge-cli`)
- **Command-Line Interface**: Ergonomic subcommands for build, test, check, graph, doctor, query, affected, and daemon management.
- **Interactive SVG DAG Visualizer**: Web-based real-time dependency graph canvas with pan/zoom, search, node focus, and critical path highlighting.
- **5-Language UI Localization**: Built-in dictionary supporting English, Vietnamese, Simplified Chinese, Traditional Chinese, and Japanese.
- **Background Daemon IPC**: Loopback TCP daemon on `127.0.0.1:9527` for instant warm graph resolutions.

---

## Language Backends

Forge includes 11 dedicated language adapters:

| Backend | Identifier | Primary Manifest | Default Compiler / Tool |
| :--- | :--- | :--- | :--- |
| **Rust** | `rust` | `Cargo.toml` | `cargo`, `rustc` |
| **C / C++** | `cc` | `CMakeLists.txt`, `Makefile` | `cmake`, `clang`, `gcc`, `msvc` |
| **Go** | `go` | `go.mod` | `go build`, `go test` |
| **TypeScript / Node** | `ts` | `package.json` | `npm`, `pnpm`, `yarn`, `bun` |
| **Python** | `py` | `pyproject.toml`, `requirements.txt` | `python -m build`, `pytest`, `uv` |
| **Java / Kotlin** | `java` | `pom.xml`, `build.gradle` | `mvn`, `gradle` |
| **.NET** | `dotnet` | `*.csproj`, `*.sln` | `dotnet build`, `dotnet test` |
| **Swift** | `swift` | `Package.swift` | `swift build`, `swift test` |
| **Dart / Flutter** | `dart` | `pubspec.yaml` | `dart compile`, `flutter build` |
| **Zig** | `zig` | `build.zig` | `zig build` |
| **Docker** | `docker` | `Dockerfile` | `docker build` |

---

## Security & Verification

- **Artifact Cryptographic Signing (`forge-signing`)**: Ed25519 digital signature generation and verification.
- **SBOM Generation**: SPDX and CycloneDX Software Bill of Materials export.
- **Vulnerability Scanner (`forge-security`)**: Automated dependency scanning with CVSS scoring and severity blocking.
- **Secrets Management (`forge-secrets`)**: HashiCorp Vault, AWS Secrets Manager, and Kubernetes Secret integration with automatic console redaction.
