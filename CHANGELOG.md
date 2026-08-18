# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of Forge build orchestration system
- Support for 10+ language backends (Rust, C/C++, Go, TypeScript, Python, Java, .NET, Swift, Dart, Zig, Docker)
- CAS (Content-Addressable Storage) artifact cache
- Distributed cluster execution with remote workers
- Virtual File System (VFS) for on-demand file streaming
- Plugin system with ScriptPlugin support (Shell, Python, Node, WASM, Lua)
- CI/CD generator for GitHub Actions, GitLab CI, CircleCI, and Bitbucket Pipelines
- Build artifact signing & verification with Ed25519
- SBOM generation (SPDX/CycloneDX formats)
- Dependency vulnerability scanner
- Build cache analytics dashboard
- Multi-platform CI matrix generator
- Build notification system (Slack/Discord/Email)
- Flaky test detection & auto-retry
- Docker image building as first-class artifacts
- Secret management integration (Vault/AWS/K8s)
- Incremental build analysis
- Build pipeline templates
- VS Code extension with status bar, problem matchers, and tree view
- Experimental "dark-arts" engines for extreme performance

### Changed
- Updated repository to Foursavage organization (foursavage-dev)
- Raised MSRV from 1.86 to 1.88

### Dependencies
- Updated ratatui 0.30.0 → 0.30.2, ratatui-crossterm 0.1.0 → 0.1.2, ratatui-widgets 0.3.0 → 0.3.2
- Updated time 0.3.45 → 0.3.55 (time-core 0.1.9, time-macros 0.2.32)
- Updated ICU provider crates 2.2.0 → 2.3.0
- Updated num-conv 0.1.0 → 0.2.2, instability 0.3.10 → 0.3.13, cargo-platform 0.3.1 → 0.3.2
- Added darling 0.24.0 for derive macros; removed strum 0.27.2 / strum_macros 0.27.2
- Resolved RUSTSEC-2026-0253 via lru 0.18.2 / ratatui-core 0.1.2

### Security
- Artifact signing with cryptographic signatures
- Dependency vulnerability scanning
- Secure secret management
- Capability-based VFS permissions
- Resolved RUSTSEC-2026-0253 (lru use-after-free, fixed in lru 0.18.2)

## [0.1.0] - 2026-08-15

### Added
- Initial public release
- Core build orchestration engine
- Cache-first incremental builds
- Multi-language backend support
- Basic CI/CD generation

[Unreleased]: https://github.com/foursavage-dev/forge-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/foursavage-dev/forge-rs/releases/tag/v0.1.0
