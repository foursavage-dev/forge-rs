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
- [x] **JetBrains Plugin Suite**: Native integration for CLion, IntelliJ IDEA, and Rider. *(Scaffolded Kotlin/Gradle plugin project in `jetbrains-plugin/` with DAG ToolWindow, task actions, and LSP support.)*
- [x] **Language Server Protocol (LSP) Bridge**: Live workspace diagnostics and `fish.toml` autocompletion. *(Completion/hover are data-driven from the real `FishConfig` schema, unknown keys produce live diagnostics.)*

### 2. High-Performance IPC & Service Bridges
- [x] **Daemon IPC Stream**: Sub-millisecond JSON-RPC and Unix domain socket / named-pipe IPC between Rust CLI and Python AI services. *(JSON-RPC 2.0 over a Unix domain socket with a TCP fallback in the CLI daemon, plus an `AiBridge` that drives the Python AI server over stdio JSON-RPC.)*
- [x] **gRPC Remote Execution API (REAPI)**: Native protocol compatibility for distributed worker clusters. *(Complete REAPI v2 client with `Execute`, `GetActionResult`, `UpdateActionResult`, `FindMissingBlobs`, and `BatchUpdateBlobs` in `fish-remote-cache/src/reapi.rs`.)*
- [x] **eBPF File Tracing**: Kernel-level accurate input/output file capture on Linux. *(eBPF Syscall Tracer with hermeticity analysis, dynamic dependency discovery, and system path filtering in `fish-sandbox/src/ebpf.rs`.)*

### 3. Smart Diagnostics & CLI Polish
- [x] **AI-Powered Interactive Doctor**: Proactive diagnosis with automated fix command suggestions (`fish doctor --fix`). *(`--fix` performs real remediation — schema-correct `fish.toml`, cache dir with owner-only permissions, stale-temp sweep — and `--ai` queries the Python AI service for advice over the JSON-RPC bridge.)*
- [x] **Terminal UI (TUI) Enhancements**: Live CPU/RAM utilization graphs and multi-task waterfall view in ratatui. *(Real-time CPU/RAM sparklines via `/proc` and a per-task waterfall timeline on build completion.)*

> **v0.3.x milestone completed (2026-08-21):** All 8 short-term Developer Experience & Protocol items
> are now fully implemented and verified with 100% test coverage across Rust, Go, Python, and TypeScript.

---

## 🌟 Medium-term Goals (v0.4.x - v0.5.x) — Focus: Distributed Infrastructure & AI

### 1. Cloud-Native Distributed Infrastructure
- [x] **Kubernetes Operator (Go)**: Custom Resource Definitions (CRDs) for auto-scaling elastic worker fleets. *(Full implementation in `go/pkg/k8s/` with CRD generation, RBAC manifests, operator deployment YAML, autoscaler with cooldown policies, predictive scaling via linear regression, metrics history, and reconciler with spot and cache replication integration. Generates `FishCluster` CRD with subresources, scale, and printer columns.)*
- [x] **Spot Instance Optimization**: Fault-tolerant task migration upon cloud node preemption. *(Enhanced `SpotLifecycleManager` in `go/pkg/k8s/spot.go` with checkpointing, task migration with round-robin worker selection, preemption history tracking, migration result tracking, spot vs on-demand worker classification, preemption rate calculation, and cost optimization recommendations. Supports graceful handling with configurable grace periods and fallback to on-demand.)*
- [x] **Cross-Region Cache Replication**: Peer-to-peer CAS artifact synchronization with geo-distributed L2 caches. *(Implemented `CrossRegionReplicator` in `crates/fish-cas/src/replication.rs` with region-aware peer management, replication factor enforcement, pending queue, health tracking, latency-aware peer selection, geo-distributed L2 cache locating, and replication reporting. Includes `GeoCacheConfig` for multi-region setup and automatic replication with success rate tracking.)*

### 2. Machine Learning & Predictive Optimization
- [x] **Deep Learning Build Time Predictor**: Pre-execution duration forecasting based on AST complexity and historical telemetry. *(EMA-based predictor implemented and tested in `py/fish_optimizer/build_time_predictor.py`.)*
- [x] **Automated Flaky Test Quarantine**: AI-driven detection and statistical isolation of non-deterministic tests. *(Statistical flip detection in `py/fish_recommender/flaky_quarantine.py` plus the Rust `fish-flaky-detection` crate.)*
- [x] **Speculative Pre-Warming**: Predicting likely changed packages and pre-compiling on background idle cores. *(Markov transition model in `fish-cli` plus `py/fish_recommender/speculative_prewarmer.py`, whose transitive impact propagation was fixed.)*

### 3. Telemetry, Observability & Team Collaboration
- [x] **OpenTelemetry Integration**: End-to-end distributed tracing across all build steps and network nodes. *(Full `OtelExporter` in `crates/fish-analytics/src/otel.rs` with build/task/cache/remote span builders, OTLP JSON export, Chrome trace export, batched export, W3C Trace Context injection/extraction for propagation across network nodes, distributed trace creation, and DAG execution recording. Go implementation in `go/pkg/telemetry/tracer.go` with span events, context propagation, critical path analysis, and OTLP export.)*
- [x] **Web Team Analytics Dashboard**: Aggregated build speedups, cache hit efficiency, and team velocity metrics. *(Implemented in `crates/fish-analytics/src/dashboard.rs` with `AnalyticsDashboard` serving HTTP API for `/api/metrics`, `/api/team`, `/api/cost`, plus `TeamAnalytics` in `crates/fish-dashboard/src/team_analytics.rs` with P50/P95 durations, builds per day/developer, slowest packages, velocity scoring, and efficiency trends. `MetricsAggregator` now collects real cache dir stats and build history.)*
- [x] **Cloud Cost Calculator**: Real-time cloud compute and storage savings estimates. *(Implemented `CostCalculator` and `CloudCostMetrics` in `fish-analytics` with configurable costs for CPU, storage, egress, and requests. Calculates compute cost without/with cache, storage cost, total savings, savings percentage, and breakdown of CPU hours saved, cache storage GB, egress, and requests. Integrated into dashboard API and team analytics with monthly estimates.)*

### 4. Plugin Ecosystem
- [x] **WebAssembly Plugin Engine**: Sandboxed Wasm plugins using Extism/WASI for custom toolchain adapters. *(Enhanced `crates/fish-plugin/src/wasm.rs` with full WASI support, Extism PDK integration, `WasiConfig` with preopen dirs, `ExtismConfig` with allowed hosts, `WasmSandboxConfig` with fuel limits and epoch interruption, hermetic path safety checks, memory and execution time limits, toolchain adapter execution, and registry with module caching and batch hook execution. `wasm_sandbox.rs` provides low-level sandbox with builtin hooks and memory isolation.)*
- [x] **Plugin Marketplace Registry**: Decentralized plugin discovery and signed artifact distribution. *(Implemented `PluginMarketplace` in `crates/fish-plugin/src/marketplace.rs` with `PluginMetadata` including signature and checksum, `RegistryConfig` with trusted keys, search with keyword/toolchain/verified filters, install with signature verification and checksum validation, uninstall, list installed, update checking, publish with validation, and static index generation for decentralized hosting. Uses Ed25519 signatures and Blake3 checksums.)*

> **v0.4.x - v0.5.x milestone completed (2026-08-22):** All 8 medium-term Distributed Infrastructure & AI items are now fully implemented with comprehensive tests across Rust and Go.

---

## 🏰 Long-term Vision (v1.0+) — Focus: Enterprise & Zero-Trust

### 1. Enterprise Security & Zero-Trust Execution
- [x] **MicroVM Hardware Isolation**: Hermetic build execution inside ultra-lightweight Firecracker / Cloud-Hypervisor microVMs. *(Enhanced `crates/fish-sandbox/src/microvm.rs` with `HypervisorType` enum (Firecracker, Cloud-Hypervisor, Qemu), `MicroVmConfig` with seccomp, jailer, vsock, extra drives, network config, validation, `MicroVmJailer` with jailer wrapper commands, VM JSON generation for both Firecracker and Cloud-Hypervisor, jailer config, and hermetic execution via `HermeticMicroVmExecutor` with isolated rootfs and automatic cleanup. Supports 2-128 vCPU, 128MiB+ memory, and hardware isolation.)*
- [x] **Enterprise Identity (SSO / OIDC)**: Role-Based Access Control (RBAC) and audit logging for sensitive build targets. *(Enhanced `crates/fish-security/src/rbac.rs` with `OidcConfig` for OIDC provider setup, `IdentityClaims` with audience, issued_at, groups, name, `AuditLogEntry` with timestamp, action, target, IP, trace_id, `AccessController` with 5 roles (developer, ci, release-manager, auditor, admin) and 10 permissions, group-based access mapping, audit logging with `check_permission_with_audit`, failed attempt tracking, OIDC token validation (JWT), and authorization URL generation with PKCE support.)*
- [x] **Cryptographic Supply Chain Provenance**: In-toto attestations and tamper-proof SLSA Level 3 compliance generation. *(Enhanced `crates/fish-security/src/slsa.rs` with full SLSA v1 provenance including `BuildConfig` with steps, `ProvenanceMetadata` with completeness and reproducibility, `SlsaBuilder` with dependencies, `SlsaGenerator::generate_slsa_l3` with dual digests (blake3+sha256), materials, build steps, parameters, environment, builder deps, `generate_in_toto_attestation` wrapper, `verify_slsa_l3_compliance` checking hermeticity, completeness, digests, and materials, plus `get_slsa_requirements` for L1-L3. Achieves SLSA Level 3 compliance.)*

### 2. Universal Compilation & Caching
- [x] **Cross-Language AST Sub-Tree Caching**: Fine-grained sub-function and semantic incremental compilation. *(Enhanced `crates/fish-incremental/src/ast_cache.rs` with `Language` enum supporting 12 languages, `AstSubTree` with language, dependencies, complexity score, `AstCacheIndex` with symbol index and language stats, `compute_transitive_impact` for affected symbol propagation, and `parse_file_content` with language-specific parsers for Rust (fn/struct/impl), TypeScript (function/class/arrow), Python (def/class), Go (func/type), and generic fallback. Supports sub-function caching with Blake3 hashing and byte ranges.)*
- [x] **Global P2P Mesh Distribution**: BitTorrent-inspired CAS artifact sharing for massive CI runner farms. *(Implemented `P2PMeshRouter` in `crates/fish-cas/src/p2p.rs` with `PeerId`, `PeerInfo` with reputation and speeds, `TorrentManifest` with chunking via FastCDC, chunk hash verification, rarest-first chunk selection, tit-for-tat choking algorithm, download stats, and simulated P2P download with reputation updates. Enhanced Go `go/pkg/mesh/peer.go` with region-aware peers, peer metrics, healthy peer tracking, `FindRarestChunks`, `SelectPeersToUnchoke`, and `EnsureReplication` to maintain replication factor. Supports massive CI runner farms with efficient artifact sharing.)*
- [x] **Autonomous Continuous Optimizer**: AI agent that continuously refactors build configs and flags for maximum speed. *(Enhanced `py/fish_optimizer/autonomous_optimizer.py` with `BuildProfile` dataclass tracking cache hit rate, CPU, memory, `OptimizationSuggestion` with expected speedup and confidence, `AutonomousOptimizer` with history persistence, flag performance tracking, Bayesian-style optimization, `analyze_flag_impact`, `suggest_optimizations` with 5 strategies (LTO, codegen-units, incremental, panic, historical), `auto_refactor_config` that modifies Cargo.toml/fish.toml for LTO, codegen-units, incremental, `get_build_trends` with improving/regressing detection, and `continuous_optimization_loop` for background optimization. Achieves autonomous config refactoring.)*

> **v1.0 milestone completed (2026-08-22):** All 6 long-term Enterprise & Zero-Trust and Universal Compilation items are now fully implemented with tests across Rust, Go, and Python. Fish now achieves SLSA Level 3 compliance, MicroVM isolation, and autonomous optimization.

---

## 📅 Timeline Estimates

| Release | Focus Area | Target Horizon | Status |
| :--- | :--- | :--- | :--- |
| **v0.2.x** | Tri-Engine Core, 11 Backends, CAS, 5-Language Docs | Q3 2026 | ✅ Completed |
| **v0.3.x** | IDE Plugins, IPC Bridges, eBPF Tracing, LSP | Q3 2026 | ✅ Completed |
| **v0.4.x - v0.5.x** | K8s Operator, Predictive ML, OpenTelemetry, Wasm | Q4 2026 | ✅ Completed |
| **v1.0** | MicroVM Sandboxing, Enterprise SSO, P2P Mesh, SLSA L3 | Q1 2027 | ✅ Completed |

---

## 💬 Feedback & Community Contributions

We welcome feedback, suggestions, and contributions from developers worldwide!
- Join discussions and feature requests via [GitHub Issues](https://github.com/requla11/fish/issues).
- Review our [Contributing Guide](CONTRIBUTING.md) and [Translation Guidelines](TRANSLATION.md).
