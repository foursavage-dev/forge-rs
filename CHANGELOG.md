# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-08-19

### Changed
- Bumped workspace version to 0.2.0.
- Massive refactoring: Replaced all `Box<dyn std::error::Error>` with robust `anyhow::Error` across 12 crates (analytics, backend-docker, docker-builder, flaky-detection, notifications, secrets, templates, worker, etc.).
- Improved cross-crate dependency version management in Cargo.toml.
- Upgraded the Dashboard web UI to include complete styling and real-time visualization layouts.
- Made all modules significantly more idiomatic and complete.

### Experimental (Dark-Arts Engines)
- **Autonomous Binary Super-Optimizer:** Synthesizes SIMD (`AVX-512`/`NEON`) instructions to aggressively vectorize basic blocks in target ELF/PE binaries.
- **Live Binary Hot-Patching:** Simulates kernel-level `ptrace` attachment and injects Trampoline `JMP` payloads directly into process memory without downtime.
- **Predictive Engine:** Rebuilt with a **Markov-Chain Speculative Pre-Compilation Engine** to learn developer workflow paths and trigger asynchronous cache warmups.
- **Work-Stealing Scheduler:** Added **Machine Learning Heuristics** with moving-average profiling and Greedy Load Balancing (LPT) with Critical-Path scoring to prioritize deep DAG bottlenecks.
- **Kernel-Bypass DMA VFS:** Implemented Lock-Free Ring-Buffers to stream massive build artifacts directly through memory, bypassing kernel disk I/O sys-calls.
- **WASM/WASI Hermetic Sandbox:** Execute external polyglot plugins in a strictly isolated environment with bounded memory pages and CPU "fuel" limits.


### Added
- Bazel-style algebraic graph query engine (`forge query` with `deps()`, `rdeps()`, `allpaths()`, `somepath()`, `filter()`)
- Pants-style automated AST dependency inference (`DependencyInferenceEngine`) for Rust, TypeScript/JavaScript, Python, and Go
- Nx-style Distributed Task Execution (DTE) bin-packing (`DteBinPacker`) for multi-agent CI balance
- Ninja-style dirty rebuild diagnostics (`DirtyExplainer`, `forge build --explain`)
- GNU Jobserver pool integration (`JobserverPool`) for global compiler thread token coordination
- Dynamic remote racing (`DynamicRacingExecutor`) racing local and remote worker execution
- Background loopback TCP build daemon (`ForgeDaemon`, `forge daemon start/status/stop`)
- Response file synthesis (`ResponseFileWriter`, `@forge_args.rsp`) for large compiler argument vectors
- Profile-Guided Optimization (PGO) orchestration (`PgoManager`, `--pgo-generate`, `--pgo-use`)
- Task pipeline topology engine (`PipelineResolver`) with cross-package `^build` dependencies
- Redesigned Stage Tree DAG visualizer for clean terminal representations
- Support for 11+ language backends (Rust, C/C++, Go, TypeScript, Python, Java, .NET, Swift, Dart, Zig, Docker)
- CAS (Content-Addressable Storage) artifact cache with Blake3 and Zstandard compression
- Distributed cluster execution with remote workers and binary streaming protocol
- Virtual File System (VFS) for on-demand file streaming
- Plugin system with ScriptPlugin support (Shell, Python, Node, WASM, Lua)
- CI/CD generator for GitHub Actions, GitLab CI, CircleCI, and Bitbucket Pipelines
- Build artifact signing & verification with Ed25519 and SPDX SBOM generation
- Dependency vulnerability scanner and HashiCorp Vault / AWS Secrets Manager integration
- Build cache analytics dashboard and flaky test detection & auto-retry

### Changed
- Updated repository to Foursavage organization (foursavage-dev)
- Raised MSRV from 1.86 to 1.88

### Dependencies
- Updated ratatui 0.30.0 → 0.30.2, ratatui-crossterm 0.1.0 → 0.1.2, ratatui-widgets 0.3.0 → 0.3.2
- Updated time 0.3.45 → 0.3.55 (time-core 0.1.9, time-macros 0.2.32)
- Updated ICU provider crates 2.2.0 → 2.3.0
- Updated num-conv 0.1.0 → 0.2.2, instability 0.3.10 → 0.3.13, cargo-platform 0.3.1 → 0.3.2
- Added darling 0.24.0 for derive macros; removed strum 0.27.2 / strum_macros 0.27.2
- Resolved RUSTSEC-2026-0253 via lru 0.18.2 / ratatui-core 0.1.2

### Security
- Artifact signing with cryptographic signatures
- Dependency vulnerability scanning
- Secure secret management
- Capability-based VFS permissions
- Resolved RUSTSEC-2026-0253 (lru use-after-free, fixed in lru 0.18.2)

## [0.1.0] - 2026-08-15

### Added
- Initial public release
- Core build orchestration engine
- Cache-first incremental builds
- Multi-language backend support
- Basic CI/CD generation

[Unreleased]: https://github.com/foursavage-dev/forge-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/foursavage-dev/forge-rs/releases/tag/v0.1.0
