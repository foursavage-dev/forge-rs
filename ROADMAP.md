# Fish Roadmap

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

This document outlines the planned development roadmap for Fish.

## Vision

Fish aims to be the most efficient, secure, and developer-friendly build orchestration system for polyglot monorepos and distributed development.

## Current Version (v0.2.x)

### Phase 1: Core Engine & Multi-Language Backends
- [x] Complete core build orchestration engine
- [x] Implement all 11 language backends (Rust, C/C++, Go, TS, Py, Java, .NET, Swift, Dart, Zig, Docker)
- [x] Add distributed worker support and cluster execution
- [x] Implement Blake3 Content-Addressable Storage (CAS) with Zstandard compression
- [x] Add CI/CD generation for GitHub, GitLab, CircleCI, Bitbucket
- [x] Implement extensible plugin system
- [x] Add security features (Ed25519 signing, SPDX SBOM, vulnerability scanning)
- [x] Create comprehensive documentation and translation framework

### Phase 2: Performance, Visualization & Developer Experience
- [x] Web-based interactive telemetry dashboard & DAG visualizer (`Fish ui`) with 5-language UI
- [x] Standard compilation database generator (`CompilationDatabase`, `compile_commands.json`)
- [x] Hermetic toolchain registry and system auto-detection (`ToolchainRegistry`)
- [x] Real-time filesystem watcher daemon with dirty target tracking (`FsWatcherDaemon`)
- [x] Algebraic graph query engine (`Fish query` with `deps()`, `rdeps()`, `allpaths()`, `somepath()`, `filter()`)
- [x] Copy-on-Write extents cloner (`KernelCowCloner`) and modern linker dispatcher (`mold`, `lld`, `msvc`)
- [x] Kernel resource governor (`KernelResourceGovernor`) for memory pressure protection
- [x] GNU Jobserver pool integration (`JobserverPool`) for global compiler thread token coordination
- [x] Dynamic remote racing (`DynamicRacingExecutor`) and DTE bin-packing (`DteBinPacker`)
- [x] Profile-Guided Optimization (PGO) automated 2-phase workflow
- [x] Comprehensive test coverage across all 34 crates (100% tests passed)

---

## Short-term Goals (v0.3.x)

### IDE & Editor Integration
- [ ] Dedicated VS Code extension with interactive DAG preview and task runner
- [ ] JetBrains plugin (CLion, IntelliJ, Rider) integration
- [ ] Language Server Protocol (LSP) workspace diagnostics bridge

### Distributed Infrastructure & Cloud
- [ ] Kubernetes operator for elastic worker auto-scaling
- [ ] gRPC Remote Execution API (REAPI) server compatibility
- [ ] Automated PEX virtual environment packaging for hermetic Python execution

## Medium-term Goals (v0.4.x - v0.5.x)

### Advanced Features
- [ ] Machine learning-based optimization
- [ ] Predictive caching
- [ ] Automated refactoring suggestions
- [ ] Dependency management integration
- [ ] Package manager integration

### Cloud Integration
- [ ] Native cloud deployment
- [ ] Kubernetes operator
- [ ] Auto-scaling workers
- [ ] Managed service offering

### Collaboration
- [ ] Real-time collaboration
- [ ] Build sharing
- [ ] Team analytics
- [ ] Cost tracking

### Ecosystem
- [ ] Plugin marketplace
- [ ] Template marketplace
- [ ] Integration marketplace
- [ ] Community plugins

## Long-term Vision (v1.0+)

### Enterprise Features
- [ ] Multi-tenant support
- [ ] SSO integration
- [ ] Advanced RBAC
- [ ] Compliance certifications
- [ ] Enterprise support

### Advanced Analytics
- [ ] Build intelligence
- [ ] Resource optimization
- [ ] Cost optimization
- [ ] Performance insights

### Ecosystem Expansion
- [ ] Additional language backends
- [ ] Custom toolchain support
- [ ] Universal build adapter
- [ ] Legacy system integration

## Experimental Features

### "Dark-Arts" Engines
These are experimental high-performance engines for extreme use cases:

- [x] Live Binary Hot-Patching
- [ ] Linker Turbo-Hijack
- [x] Speculative Markov Pre-Compilation
- [x] WASM/WASI Hermetic Sandbox
- [ ] Pre-Warmed Compiler Daemon Pool
- [x] In-Process Micro-JIT Synthesis
- [x] Autonomous Binary Super-Optimizer
- [x] Kernel-Bypass DMA Ring-Buffer VFS

## Community Goals

- [ ] Active community of contributors
- [ ] Comprehensive documentation
- [ ] Tutorial and guide ecosystem
- [ ] Integration examples
- [ ] Conference talks and presentations

## Feedback and Contributions

We welcome feedback on our roadmap. Please:
- Open an issue for feature requests
- Join our Discord community
- Participate in discussions
- Contribute to the project

## Timeline Estimates

- **v0.1.x**: Q3 2026 (Current)
- **v0.2.x**: Q4 2026
- **v0.3.x**: Q1 2027
- **v1.0**: Q2 2027

*Note: Timelines are estimates and may change based on community feedback and priorities.*
