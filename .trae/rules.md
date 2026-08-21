# Trae AI Rules for Fish Project

## Engineering Standards:
- Build engine: Fish polyglot build orchestrator.
- Rust Edition: 2024 (MSRV 1.88+).
- Go Services: Go 1.22+ in `go/`.
- Python AI: Python 3.11+ in `py/`.
- Extension: TypeScript VS Code extension in `vscode-extension/`.

## Key Invariants:
1. Always implement production-ready real code, never return mock success.
2. Ensure strict Cross-platform path handling (`/` normalization for cache keys).
3. Keep all tests passing: `cargo test --workspace`, `go test ./...`, `python -m unittest`.
