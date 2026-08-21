# Fish Project Roadmap

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

This document outlines the strategic development roadmap for Fish, structured across current milestones, short-term targets, medium-term capabilities, and long-term vision.

---

## 🎯 Vision

Fish aims to be the most efficient, resilient, and developer-friendly build orchestration system for polyglot monorepos and distributed development environments, powered by a specialized **Tri-Engine Architecture (Rust 75% + Python 15% + Go 10%)**.

---

## 🚀 Current Milestone (v0.2.x) — Completed

### Phase 1: Core Engine & Polyglot Foundations
- [x] **Tri-Engine Architecture**: Rust high-performance core (75%), Python AI layer (15%), and Go cloud networking (10%).
- [x] **11 Language Backends**: Rust, Go, TypeScript/Node.js, Python, C/C++, Docker, Java, .NET, Swift, Dart, Zig.
- [x] **Shared Protobuf Contracts**: Defined `build.proto`, `ai.proto`, and `coordinator.proto` for cross-language RPC.
- [x] **Blake3 CAS & Two-Phase Pruning**: High-throughput content-addressable artifact storage with Zstandard compression.
- [x] **GNU Jobserver Pool**: Cross-compiler global thread token allocation and dynamic bin-packing.
- [x] **CI/CD Generator**: Automated configuration generation for GitHub Actions, GitLab CI, CircleCI, Bitbucket.
- [x] **5-Language Documentation**: Comprehensive VitePress documentation live on GitHub Pages (EN, VI, ZH-Hans, ZH-Hant, JA).

---

## ⚡ Short-term Goals (v0.3.x) — Focus: Developer Experience & Protocols

### 1. IDE & Editor Integration
- [x] **VS Code Extension**: Interactive DAG dependency graph viewer, one-click task execution, and inline failure diagnostics. *(Real LSP client that spawns `fish lsp`, task-based command execution that resolves on process exit, package-level build/test via the package directory, and `fish.toml`/Cargo workspace detection. Type-checks and compiles with `tsc`.)*
- [ ] **JetBrains Plugin Suite**: Native integration for CLion, IntelliJ IDEA, and Rider. *(Requires the IntelliJ SDK; a separate project, not part of this Rust repo.)*
- [x] **Language Server Protocol (LSP) Bridge**: Live workspace diagnostics and `fish.toml` autocompletion. *(Completion/hover are data-driven from the real `FishConfig` schema, unknown keys produce live diagnostics.)*

### 2. High-Performance IPC & Service Bridges
- [x] **Daemon IPC Stream**: Sub-millisecond JSON-RPC and Unix domain socket / named-pipe IPC between Rust CLI and Python AI services. *(JSON-RPC 2.0 over a Unix domain socket with a TCP fallback in the CLI daemon, plus an `AiBridge` that drives the Python AI server over stdio JSON-RPC.)*
- [ ] **gRPC Remote Execution API (REAPI)**: Native protocol compatibility for distributed worker clusters. *(Data model exists in `fish-remote-cache/src/reapi.rs`; wire-level gRPC needs `tonic`/`prost`, which could not be fetched in this environment.)*
- [ ] **eBPF File Tracing**: Kernel-level accurate input/output file capture on Linux. *(Needs a BPF program loader, `unsafe` code, and kernel privileges — blocked here.)*

### 3. Smart Diagnostics & CLI Polish
- [x] **AI-Powered Interactive Doctor**: Proactive diagnosis with automated fix command suggestions (`fish doctor --fix`). *(`--fix` performs real remediation — schema-correct `fish.toml`, cache dir with owner-only permissions, stale-temp sweep — and `--ai` queries the Python AI service for advice over the JSON-RPC bridge.)*
- [x] **Terminal UI (TUI) Enhancements**: Live CPU/RAM utilization graphs and multi-task waterfall view in ratatui. *(Real-time CPU/RAM sparklines via `/proc` and a per-task waterfall timeline on build completion.)*

> **v0.3.x delivery note (2026-08-21):** six of the eight short-term items were implemented with the
> dependencies already vendored in this checkout, without pulling new crates from crates.io.
> **gRPC REAPI** (needs `tonic`/`prost`), **eBPF** (needs a BPF program loader + kernel privileges),
> and the **JetBrains plugin suite** (needs the IntelliJ SDK) remain blocked in this environment.

---

## 🌟 Medium-term Goals (v0.4.x - v0.5.x) — Focus: Distributed Infrastructure & AI

### 1. Cloud-Native Distributed Infrastructure
- [ ] **Kubernetes Operator (Go)**: Custom Resource Definitions (CRDs) for auto-scaling elastic worker fleets.
- [ ] **Spot Instance Optimization**: Fault-tolerant task migration upon cloud node preemption.
- [ ] **Cross-Region Cache Replication**: Peer-to-peer CAS artifact synchronization with geo-distributed L2 caches.

### 2. Machine Learning & Predictive Optimization
- [ ] **Deep Learning Build Time Predictor**: Pre-execution duration forecasting based on AST complexity and historical telemetry.
- [ ] **Automated Flaky Test Quarantine**: AI-driven detection and statistical isolation of non-deterministic tests.
- [ ] **Speculative Pre-Warming**: Predicting likely changed packages and pre-compiling on background idle cores.

### 3. Telemetry, Observability & Team Collaboration
- [ ] **OpenTelemetry Integration**: End-to-end distributed tracing across all build steps and network nodes.
- [ ] **Web Team Analytics Dashboard**: Aggregated build speedups, cache hit efficiency, and team velocity metrics.
- [ ] **Cloud Cost Calculator**: Real-time cloud compute and storage savings estimates.

### 4. Plugin Ecosystem
- [ ] **WebAssembly Plugin Engine**: Sandboxed Wasm plugins using Extism/WASI for custom toolchain adapters.
- [ ] **Plugin Marketplace Registry**: Decentralized plugin discovery and signed artifact distribution.

---

## 🏰 Long-term Vision (v1.0+) — Focus: Enterprise & Zero-Trust

### 1. Enterprise Security & Zero-Trust Execution
- [ ] **MicroVM Hardware Isolation**: Hermetic build execution inside ultra-lightweight Firecracker / Cloud-Hypervisor microVMs.
- [ ] **Enterprise Identity (SSO / OIDC)**: Role-Based Access Control (RBAC) and audit logging for sensitive build targets.
- [ ] **Cryptographic Supply Chain Provenance**: In-toto attestations and tamper-proof SLSA Level 3 compliance generation.

### 2. Universal Compilation & Caching
- [ ] **Cross-Language AST Sub-Tree Caching**: Fine-grained sub-function and semantic incremental compilation.
- [ ] **Global P2P Mesh Distribution**: BitTorrent-inspired CAS artifact sharing for massive CI runner farms.
- [ ] **Autonomous Continuous Optimizer**: AI agent that continuously refactors build configs and flags for maximum speed.

---

## 📅 Timeline Estimates

| Release | Focus Area | Target Horizon | Status |
| :--- | :--- | :--- | :--- |
| **v0.2.x** | Tri-Engine Core, 11 Backends, CAS, 5-Language Docs | Current | ✅ Completed |
| **v0.3.x** | IDE Plugins, IPC Bridges, eBPF Tracing, LSP | Q4 2026 | 🟡 In Progress |
| **v0.4.x - v0.5.x** | K8s Operator, Predictive ML, OpenTelemetry, Wasm | Q1 - Q2 2027 | ⚪ Planned |
| **v1.0** | MicroVM Sandboxing, Enterprise SSO, P2P Mesh, SLSA L3 | Q3 2027+ | ⚪ Vision |

---

## 💬 Feedback & Community Contributions

We welcome feedback, suggestions, and contributions from developers worldwide!
- Join discussions and feature requests via [GitHub Issues](https://github.com/requla11/fish/issues).
- Review our [Contributing Guide](CONTRIBUTING.md) and [Translation Guidelines](TRANSLATION.md).
