# Claude Code Guidelines for Fish

## Build & Test Commands
- Build CLI: `cargo build -p fish-cli`
- Workspace Tests: `cargo test --workspace`
- Clippy: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Format: `cargo fmt --all -- --check`
- Go Tests: `cd go && go test -v ./...`
- Python Tests: `cd py && python -m unittest discover tests`

## Architecture & Code Standards
- Polyglot DAG engine with BLAKE3 fingerprinting and ZSTD CAS storage.
- Real production logic only — no fabricated `Ok(())` stubs.
- Cross-platform path normalization using `/` for cache keys.
