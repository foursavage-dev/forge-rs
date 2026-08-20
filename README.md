# Fish

[![CI](https://github.com/requla11/fish/actions/workflows/dogfood.yaml/badge.svg)](https://github.com/requla11/fish/actions/workflows/dogfood.yaml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

Fish is a Rust build-orchestration experiment for projects that use more than
one toolchain. It discovers supported projects, constructs a dependency graph,
and runs build, check, and test work with local caching and parallel scheduling.

Fish does not replace compilers or package managers. It coordinates tools such
as Cargo, Go, Node.js, Python, C/C++, Docker, and other supported backends.

> Status: pre-1.0. The CLI and configuration may change. Treat distributed,
> remote-cache, and experimental features as opt-in and validate them in your
> own environment before relying on them in CI.

## What works today

- Rust workspace discovery and graph-based `build`, `check`, and `test`.
- Backends for Rust plus C/C++, Go, TypeScript/JavaScript, Python, Java,
  .NET, Swift, Dart, Zig, Docker, and script plugins.
- Local fingerprint and content-addressable artifact storage.
- Graph output (`tree`, `json`, and `dot`), file watching, affected-project
  detection, profiles, a terminal UI, and CI configuration generation.
- Optional remote cache/worker, sandbox, signing, and experimental modules.

The source tree contains the authoritative list of commands and supported
options. Run `Fish --help` and `Fish <command> --help` for your installed
version.

## Branch Policy

Fish uses two main branches:

- **`main`** - The stable branch and the primary source of code for the
  project. Code in `main` should be tested and considered stable.
- **`dev`** - The development and experimental branch. New features, changes,
  fixes, and other experimental code are developed and tested here first.

Changes should **not be merged directly into `main`** during normal
development. Instead, changes are developed and tested on `dev`. Once the
changes have been verified and are considered stable, they can be merged from
`dev` into `main`.

In short:

```text
dev
  ↓
  ↓  develop + test
  ↓
[verified / stable]
  ↓
  ↓  merge
  ↓
main
```

> **Important:** `main` is intended to contain stable code, while `dev` may
> contain unfinished, experimental, or potentially unstable changes.

## Install

### From source

```bash
cargo install --path crates/fish-cli
```

### Development checkout

```bash
git clone https://github.com/requla11/fish.git
cd fish
cargo build -p fish-cli
```

The project requires Rust 1.88 or later (MSRV 1.88).

## Quick start

Build a supported project from its root:

```bash
fish build
fish test
fish graph --format dot
```

Useful variants:

```bash
# Select parallelism and write a trace profile.
fish build --jobs 8 --profile build-trace.json

# Inspect the detected projects and their dependencies.
fish graph --format tree

# Rebuild when source files change.
fish watch --mode test

# Launch the interactive Web Dashboard & DAG visualizer.
fish ui --port 3000 --open

# See the local cache's size and record count.
fish cache stats
```

Fish stores its local cache in `~/.fish/cache` by default. Set
`FISH_CACHE_DIR` or pass `--cache-dir <path>` to use a project- or
CI-specific location.

See [DEVELOPMENT.md](DEVELOPMENT.md) for local development and
[ARCHITECTURE.md](ARCHITECTURE.md) for the workspace design.

## Commands

| Command | Purpose |
| --- | --- |
| `fish init` | Initialize Fish configuration and generate task definitions for detected project languages. |
| `fish build`, `check`, `test` | Execute work discovered from the project graph (supports `--explain`, `--pgo-generate`, `--pgo-use`). |
| `fish ui` | Start the interactive Web Dashboard & DAG visualizer with 5-language telemetry. |
| `fish query <EXPR>` | Query graph algebra: `deps(...)`, `rdeps(...)`, `allpaths(...)`, `somepath(...)`, `filter(...)`. |
| `fish daemon` | Manage background build daemon (`start`, `status`, `stop`) for sub-millisecond warm builds. |
| `fish run` | Build and run a selected Rust package or binary. |
| `fish graph` | Print the graph as stage trees, JSON, or DOT. |
| `fish watch` | Re-run build, check, or test after relevant file changes. |
| `fish affected --since REV` | Limit work to projects changed since a revision. |
| `fish cache` | Inspect, prune, and manage the local cache and CAS. |
| `fish ci init` / `export` | Generate CI configurations for GitHub, GitLab, CircleCI, Bitbucket. |
| `fish doctor` | Check local toolchain readiness and diagnose system environment. |
| `fish worker` / `cache-server` | Start optional remote-execution services. |

Some commands require a corresponding toolchain on `PATH`. `fish doctor` is a
good first check when setting up a machine.

## High-Performance Capabilities

- **Interactive SVG DAG Web Visualizer**: Real-time dependency graph canvas with pan/zoom, node focus, critical path highlights, and 5-language UI.
- **Compilation Database Generator**: Generates standard `compile_commands.json` for Clangd, LSP, and IDEs across C/C++, Rust, and Go.
- **Hermetic Toolchain Registry**: Isolated toolchain management and environment isolation with system toolchain auto-detection.
- **Real-Time Filesystem Watcher**: Background daemon with dirty node tracking and hot graph cache pre-warming.
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
  fish-core/         project discovery, manifest model, and DAG merger
  fish-graph/        dependency graph, topological sort, and query algebra
  fish-executor/     process execution, middleware chain, and response files
  fish-scheduler/    parallel scheduling, jobserver pool, racing, and DTE bin-packing
  fish-cache/        fingerprint cache and two-phase pruning
  fish-cas/          content-addressable artifact storage with ZSTD compression
  fish-incremental/  change detection, AST inference, and dirty explainer
  fish-backend-*/    11+ language and toolchain adapters
  fish-worker/       distributed execution server and streaming VFS protocol
  fish-remote-cache/ gRPC/HTTP remote cache server
  fish-security/     multi-layer security, secret redaction, and CVE scanner
  fish-signing/      Ed25519 cryptographic artifact signing and SPDX SBOM
  fish-cli/          command-line application, daemon IPC, and terminal rendering
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

## AI Agent Guide

For AI coding agents working with this project, please refer to [AGENTS.md](AGENTS.md) for guidance on context reading order, coding patterns, and workflow instructions. The comprehensive step-by-step workflow is available in [docs/AI_AGENT_WORKFLOW.md](docs/AI_AGENT_WORKFLOW.md).

## License

Fish is licensed under the [MIT License](LICENSE).
