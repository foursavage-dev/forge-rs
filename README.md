# Forge

[![CI](https://github.com/foursavage-dev/forge-rs/actions/workflows/dogfood.yml/badge.svg)](https://github.com/foursavage-dev/forge-rs/actions/workflows/dogfood.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

Forge is a Rust build-orchestration experiment for projects that use more than
one toolchain. It discovers supported projects, constructs a dependency graph,
and runs build, check, and test work with local caching and parallel scheduling.

Forge does not replace compilers or package managers. It coordinates tools such
as Cargo, Go, Node.js, Python, C/C++, Docker, and other supported backends.

> Status: pre-1.0. The CLI and configuration may change. Treat distributed,
> remote-cache, and experimental features as opt-in and validate them in your
> own environment before relying on them in CI.


### 🧪 "Dark-Arts" Experimental Engines
Forge ships with experimental, highly-optimized performance engines for specific extreme use-cases:
- **Predictive Engine (Markov-Chain):** Learns developer patterns to pre-compile and warm up cache hits before files are even saved.
- **ML-Aware Work-Stealing:** Uses historical tracing and dependency tail length scoring to schedule threads with zero starvation.
- **Autonomous Binary Super-Optimizer:** Synthesizes `AVX-512`/`NEON` instructions directly into compiled ELF binaries via basic-block vectorization.
- **Live Binary Hot-Patching:** Injects trampoline payloads and live-patches C/Rust code in running processes via simulated memory injection.
- **Kernel-Bypass DMA Ring-Buffer VFS:** Streams artifacts directly through zero-copy memory ring-buffers, entirely bypassing disk OS sys-calls.
- **WASM/WASI Hermetic Sandbox:** Runs polyglot plugins securely via strict memory/CPU instruction bounds ("fuel limits").

## What works today

- Rust workspace discovery and graph-based `build`, `check`, and `test`.
- Backends for Rust plus C/C++, Go, TypeScript/JavaScript, Python, Java,
  .NET, Swift, Dart, Zig, Docker, and script plugins.
- Local fingerprint and content-addressable artifact storage.
- Graph output (`tree`, `json`, and `dot`), file watching, affected-project
  detection, profiles, a terminal UI, and CI configuration generation.
- Optional remote cache/worker, sandbox, signing, and experimental modules.

The source tree contains the authoritative list of commands and supported
options. Run `forge --help` and `forge <command> --help` for your installed
version.

## Branch Policy

Forge uses two main branches:

- **`main`** — The stable branch and the primary source of code for the
  project. Code in `main` should be tested and considered stable.
- **`dev`** — The development and experimental branch. New features, changes,
  fixes, and other experimental code are developed and tested here first.

Changes should **not be merged directly into `main`** during normal
development. Instead, changes are developed and tested on `dev`. Once the
changes have been verified and are considered stable, they can be merged from
`dev` into `main`.

In short:

```text
dev
 │
 │  develop + test
 ▼
[verified / stable]
 │
 │  merge
 ▼
main
```

> **Important:** `main` is intended to contain stable code, while `dev` may
> contain unfinished, experimental, or potentially unstable changes.

## Install

### From source

```bash
cargo install --path crates/forge-cli
```

### Development checkout

```bash
git clone https://github.com/foursavage-dev/forge-rs.git
cd forge-rs
cargo build -p forge-cli
```

The project requires Rust 1.85 or later.

## Quick start

Build a supported project from its root:

```bash
forge build
forge test
forge graph --format dot
```

Useful variants:

```bash
# Select parallelism and write a trace profile.
forge build --jobs 8 --profile build-trace.json

# Inspect the detected projects and their dependencies.
forge graph --format tree

# Rebuild when source files change.
forge watch --mode test

# See the local cache's size and record count.
forge cache stats
```

Forge stores its local cache in `~/.forge/cache` by default. Set
`FORGE_CACHE_DIR` or pass `--cache-dir <path>` to use a project- or
CI-specific location.

See [DEVELOPMENT.md](DEVELOPMENT.md) for local development and
[ARCHITECTURE.md](ARCHITECTURE.md) for the workspace design.

## Commands

| Command | Purpose |
| --- | --- |
| `forge build`, `check`, `test` | Execute work discovered from the project graph (supports `--explain`, `--pgo-generate`, `--pgo-use`). |
| `forge query <EXPR>` | Query graph algebra: `deps(...)`, `rdeps(...)`, `allpaths(...)`, `somepath(...)`, `filter(...)`. |
| `forge daemon` | Manage background build daemon (`start`, `status`, `stop`) for sub-millisecond warm builds. |
| `forge run` | Build and run a selected Rust package or binary. |
| `forge graph` | Print the graph as stage trees, JSON, or DOT. |
| `forge watch` | Re-run build, check, or test after relevant file changes. |
| `forge affected --since REV` | Limit work to projects changed since a revision. |
| `forge cache` | Inspect, prune, and manage the local cache and CAS. |
| `forge ci init` / `export` | Generate a CI configuration. |
| `forge doctor` | Check local toolchain readiness. |
| `forge worker` / `cache-server` | Start optional remote-execution services. |

Some commands require a corresponding toolchain on `PATH`. `forge doctor` is a
good first check when setting up a machine.

## High-Performance Capabilities

- **Hermetic CAS & ZSTD Deduplication**: Blake3 content-addressable storage with tiered L1/L2 composite caching.
- **GNU Jobserver Pool**: Token-based concurrency control across sub-processes preventing CPU thrashing.
- **Dynamic Remote Racing**: Races local execution against remote cluster workers, cancelling the slower one.
- **Distributed Task Execution (DTE)**: LPT bin-packing algorithm balancing CI workloads across workers.
- **AST Dependency Inference**: Auto-detects dependencies across Rust, TS/JS, Python, and Go source trees.
- **Graph Query Algebraic Engine**: Bazel-style `deps()`, `rdeps()`, `allpaths()`, and `filter()` queries.
- **Dirty Rebuild Diagnostics**: Detailed explanations for why a target is rebuilt (`--explain`).
- **Profile-Guided Optimization (PGO)**: Automated instrumentation, raw profile merging, and optimized compilation.

## Workspace layout

```text
crates/
  forge-core/         project discovery, manifest model, and DAG merger
  forge-graph/        dependency graph, topological sort, and query algebra
  forge-executor/     process execution, middleware chain, and response files
  forge-scheduler/    parallel scheduling, jobserver pool, racing, and DTE bin-packing
  forge-cache/        fingerprint cache and two-phase pruning
  forge-cas/          content-addressable artifact storage with ZSTD compression
  forge-incremental/  change detection, AST inference, and dirty explainer
  forge-backend-*/    11+ language and toolchain adapters
  forge-worker/       distributed execution server and streaming VFS protocol
  forge-remote-cache/ gRPC/HTTP remote cache server
  forge-security/     multi-layer security, secret redaction, and CVE scanner
  forge-signing/      Ed25519 cryptographic artifact signing and SPDX SBOM
  forge-cli/          command-line application, daemon IPC, and terminal rendering
examples/             sample projects
docs/                 additional documentation
```

## Develop and verify

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.
Security reports are handled under [SECURITY.md](SECURITY.md).

## License

Forge is licensed under the [MIT License](LICENSE).