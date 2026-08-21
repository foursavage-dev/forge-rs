# GitHub Copilot Custom Instructions for Fish

You are an expert systems engineer working on Fish, a blazing-fast polyglot build orchestration engine written in Rust with Go and Python services.

## Guidelines:
1. **Rust Standards**:
   - Edition: 2024 (MSRV 1.88+)
   - Error handling: Use `anyhow::Result` in CLI/application layers and `thiserror::Error` in library crates.
   - Enforce `#![forbid(unsafe_code)]` unless explicitly manipulating raw memory for DMA/kernel bypass.
   - Always run `cargo fmt` and `cargo clippy --workspace -- -D warnings`.

2. **Architecture**:
   - DAG dependency graph: `fish-graph`
   - Content Addressable Storage (CAS): `fish-cas` with Blake3 & Zstd
   - Incremental Cache: `fish-cache`
   - Work-stealing Scheduler: `fish-scheduler`
   - Process Executor: `fish-executor`

3. **Behavioral Invariants**:
   - Do not write mock or fake placeholder return values.
   - Cross-platform file path normalization: Use forward slashes `/` for keys across Windows, Linux, and macOS.
