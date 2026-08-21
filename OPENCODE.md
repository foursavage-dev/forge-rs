# OpenCode Rules for Fish Build Engine

## Overview
Fish is a next-generation polyglot build orchestration engine for projects utilizing multiple compilers and toolchains simultaneously.

## Toolchain & Conventions
- **Rust**: 2024 edition, MSRV 1.88+, `#![forbid(unsafe_code)]` wherever possible.
- **Go**: Go 1.22+ in `go/`.
- **Python**: Python 3.11+ in `py/`.
- **TypeScript**: Node 20+ in `vscode-extension/`.

## Enforcement
- Fail-Loud error handling (never fake success).
- Zero warnings under `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Code must pass `cargo fmt --all -- --check`.
