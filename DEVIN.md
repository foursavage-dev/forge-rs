# Devin AI & Devin IDE Instructions for Fish

## Project Summary
Fish is a polyglot build orchestration system written in Rust 2024 with Go distributed networking services and Python AI optimization services.

## Development Environment & Toolchains
- **Rust**: 1.88+ (`cargo`)
- **Go**: 1.22+ (`go`)
- **Python**: 3.11+ (`python`)
- **Node.js**: 20+ (`npm`)

## Critical Invariants & Rules
1. **Branch Workflow**: All active development takes place on the `dev` branch. Never push untested code to `main`.
2. **Quality & Validation**:
   - Every Rust code change must satisfy:
     - `cargo fmt --all -- --check`
     - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
     - `cargo test --workspace`
   - Every Go change must satisfy `go test -v ./...` in `go/`.
   - Every Python change must satisfy `python -m unittest discover tests` in `py/`.
   - Every VS Code extension change must satisfy `npm run compile && npm run lint` in `vscode-extension/`.
3. **No Mocks in Production Logic**: All components in `crates/` must be production-ready real implementations.

## Key Directories
- `crates/fish-core/`: Workspace discovery, manifest models, and toolchains.
- `crates/fish-graph/`: Dependency DAG and memoized topological traversals.
- `crates/fish-scheduler/`: Chase-Lev work stealing, jobserver, and execution pool.
- `crates/fish-cas/`: Content-Addressable Storage with Blake3 & Zstd.
- `crates/fish-cache/`: Tiered L1/L2 fingerprint cache.
- `crates/fish-cli/`: CLI binary, JSON-RPC daemon, TUI, and LSP server.
- `go/`: Distributed coordinator and worker gateway.
- `py/`: AI diagnostics, AST remediation, and risk prediction.
- `vscode-extension/`: Official IDE extension.
