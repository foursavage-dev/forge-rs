# Fish Roadmap — Status Audit

> A factual inventory that maps every item in `ROADMAP.md` to the current state of the code
> (at commit `1eb2e49` plus the uncommitted changes on the `arena/01a02246-fish` branch).
>
> Legend:
> - ✅ **Real** — working code with tests.
> - 🟡 **Partial** — code exists but is incomplete or not wired into the main flow.
> - ⚠️ **Stub / fake** — code that fakes success or is only a skeleton; needs a real implementation or must fail loudly.
> - ❌ **Missing** — does not exist.

---

## 🚀 v0.2.x — Current Milestone (self-reported "Completed")

| Roadmap item | Status | Notes |
| :--- | :---: | :--- |
| Tri-Engine Architecture (Rust/Python/Go) | ✅ | `crates/` (Rust core), `py/` (AI layer), `go/` (cloud/network) all exist |
| 11 Language Backends | ✅ | `fish-backend-{rust,cc,go,ts,py,java,dotnet,swift,dart,zig,docker}` — real, though minimal |
| Shared Protobuf Contracts | ✅ | `proto/fish/v1/{build,ai,coordinator}.proto` |
| Blake3 CAS & Two-Phase Pruning | ✅ | `fish-cas` + `fish-cache::prune` (real age + capacity phases) |
| GNU Jobserver Pool | ✅ | `fish-scheduler/src/jobserver_pool.rs` (wraps the `jobserver` crate) |
| CI/CD Generator | ✅ | `fish-ci-generator` — 5 platforms (GitHub/GitLab/CircleCI/Bitbucket/Azure) |
| 5-Language Docs | ✅ | `docs/` (EN, VI, ZH-Hans, ZH-Hant, JA) |

---

## ⚡ v0.3.x — Short-term (self-reported "In Progress")

| Roadmap item | Status | Notes |
| :--- | :---: | :--- |
| VS Code Extension | ✅ | `vscode-extension/` — real LSP client (spawns `fish lsp`), task-based command execution, package-level build/test, `fish.toml` detection; `tsc` type-checks and compiles cleanly |
| JetBrains Plugin Suite | ❌ | Does not exist (needs IntelliJ SDK — separate project) |
| LSP Bridge | ✅ | `fish-cli/src/commands/lsp.rs` — completion/hover data-driven from the real `FishConfig` schema + live unknown-key diagnostics (minimal LSP: no references/definition/rename) |
| Daemon IPC Stream | ✅ | `fish-cli/src/daemon.rs` — JSON-RPC 2.0 over Unix domain socket with TCP fallback + `ai_bridge.rs` that drives the Python AI server over stdio JSON-RPC |
| gRPC REAPI | 🟡 | `fish-remote-cache/src/reapi.rs` — serde structs + in-memory; **no wire-level gRPC** (blocked: needs `tonic`/`prost`) |
| eBPF File Tracing | ⚠️ | `fish-sandbox/src/ebpf.rs` — defines events + an "enabled" flag; **no real eBPF capture** (blocked: needs BPF loader + kernel privileges) |
| AI-Powered Interactive Doctor | ✅ | `fish-cli/src/commands/doctor.rs` — `--fix` performs real remediation; `--ai` queries the Python AI service (`doctor_advice`) over the JSON-RPC bridge |
| TUI Enhancements | ✅ | `fish-cli/src/tui.rs` — real-time CPU/RAM sparklines via `/proc` + per-task waterfall timeline on completion |

---

## 🌟 v0.4.x - v0.5.x — Medium-term (self-reported "Planned")

| Roadmap item | Status | Notes |
| :--- | :---: | :--- |
| Kubernetes Operator (Go) | 🟡 | `go/pkg/k8s/` — the autoscaler math is real, but it is in-memory logic; **no client-go/CRD integration** |
| Spot Instance Optimization | 🟡 | `go/pkg/k8s/spot.go` — basic logic |
| Cross-Region Cache Replication | 🟡 | `go/pkg/mesh/peer.go` + `fish-remote-cache/src/p2p_lan.rs` — LAN broadcast only, no geo L2 |
| DL Build Time Predictor | ✅ | `py/fish_optimizer/build_time_predictor.py` — real EMA model |
| Automated Flaky Test Quarantine | 🟡 | `fish-flaky-detection` (detector is real; `retry` was a stub → now fails loudly) + `py/fish_recommender/flaky_quarantine.py` |
| Speculative Pre-Warming | 🟡 | `fish-cli/src/predictive.rs` (real Markov model) + `py/.../speculative_prewarmer.py` |
| OpenTelemetry Integration | ❌ | No `opentelemetry`/`tracing` dependency anywhere in the workspace |
| Web Team Analytics Dashboard | 🟡 | `fish-dashboard` crate + `py/fish_analytics` — end-to-end wiring unverified |
| Cloud Cost Calculator | ❌ | Does not exist |
| Wasm Plugin Engine | ⚠️ | `wasm_sandbox.rs` (header validator is real; execution was fake → now fails loudly) + `fish-plugin/wasm.rs` |
| Plugin Marketplace Registry | ❌ | Does not exist |

---

## 🏰 v1.0 — Long-term Vision

| Roadmap item | Status | Notes |
| :--- | :---: | :--- |
| MicroVM Hardware Isolation | ⚠️ | `fish-sandbox/src/microvm.rs` — only generates a JSON config; **no Firecracker/Cloud-Hypervisor** |
| Enterprise Identity (SSO/OIDC) + RBAC | 🟡 | `fish-security/src/rbac.rs` has real RBAC logic; **no OIDC/SSO** |
| SLSA Level 3 Provenance | 🟡 | `fish-cli/src/attestation.rs` (real blake3 hashing + merkle root) + `fish-security/src/slsa.rs`; the "L3" claim is optimistic |
| Cross-Language AST Sub-Tree Caching | ❌ | `fish-incremental/src/ast_cache.rs` is just a name map, not real AST caching |
| Global P2P Mesh Distribution | 🟡 | LAN-only; not BitTorrent-inspired |
| Autonomous Continuous Optimizer | 🟡 | `py/fish_optimizer/autonomous_optimizer.py` — heuristic logic |

---

## Summary

| Category | Count |
| :--- | :---: |
| ✅ Real | 13 |
| 🟡 Partial | 11 |
| ⚠️ Stub / fake | 3 |
| ❌ Missing | 5 |

**Conclusion:** the foundation (backends, CAS, jobserver, CI generation, protobuf, docs, Python predictor)
is real and usable. Most of the remaining items are "skeleton + partial" or "stub" rather than "completely
missing". Therefore "completing the roadmap" is really a long series of jobs — implement each feature for
real and wire it into the main flow — each item being its own project rather than a single batch of work.
