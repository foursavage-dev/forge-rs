# Crates Workspace Architecture (`crates/`)

Fish is composed of 36 modular Rust crates structured into clear architectural tiers.

## Core Tiers
1. **Foundation Tier**:
   - `fish-core`: Manifest models, configuration, project discovery.
   - `fish-graph`: DAG models, lockless topological sort, query algebra.
   - `fish-executor`: Child process management, response files, middleware chains.
2. **Storage & Caching Tier**:
   - `fish-cas`: Content-Addressable Storage with ZSTD and FastCDC chunking.
   - `fish-cache`: Two-phase fingerprint cache and garbage collection.
   - `fish-remote-cache`: REAPI v2 client and TCP streaming cache.
3. **Execution & Scheduling Tier**:
   - `fish-scheduler`: Dynamic Lookahead Critical-Path scheduler, Chase-Lev work-stealing, and GNU Jobserver tokens.
   - `fish-worker`: Remote worker cluster execution and daemon loopback.
   - `fish-sandbox`: Linux eBPF syscall tracer and WASM hermetic isolation.
4. **Toolchains & Backends (11 Crates)**:
   - `fish-backend-rust`, `fish-backend-cc`, `fish-backend-go`, `fish-backend-ts`, `fish-backend-py`, `fish-backend-docker`, `fish-backend-java`, `fish-backend-dotnet`, `fish-backend-swift`, `fish-backend-dart`, `fish-backend-zig`.
5. **Security & Diagnostics Tier**:
   - `fish-security`, `fish-signing`, `fish-secrets`, `fish-flaky-detection`, `fish-notifications`, `fish-analytics`, `fish-templates`, `fish-docker-builder`, `fish-incremental`, `fish-multiplatform`, `fish-installer`.
6. **Application Tier**:
   - `fish-cli`: Unified command-line interface, TUI dashboard, and LSP server.
