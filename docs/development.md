# Fish Development Guide

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

This guide provides detailed information for developers working on the Fish codebase.

## Prerequisites

- Rust 1.88 or later (MSRV 1.88)
- Git
- Text editor / IDE (VS Code recommended)
- Docker (optional, for testing containerized workloads)

## Setting Up

```bash
# Clone the repository
git clone https://github.com/requla11/fish.git
cd fish

# Build the CLI
cargo build -p fish-cli

# Run tests across workspace
cargo test --workspace
```

## Workspace Layout

- `crates/fish-core`: Project discovery, manifest parsing, compilation database.
- `crates/fish-graph`: DAG construction, topological sort, graph query algebra.
- `crates/fish-executor`: Async process execution, response files, fast CoW cloning.
- `crates/fish-scheduler`: Work-stealing scheduler, kernel resource governor, GNU jobserver.
- `crates/fish-cache` & `fish-cas`: Fingerprinting, Content-Addressable Storage with Zstd.
- `crates/fish-backend-*`: Language backends for 11+ toolchains.
- `crates/fish-cli`: Command-line interface and interactive web dashboard.

## Code Standards & Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```
