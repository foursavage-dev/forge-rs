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

### Security
- Artifact signing with cryptographic signatures
- Dependency vulnerability scanning
- Secure secret management
- Capability-based VFS permissions

## [0.1.0] - 2026-08-15

### Added
- Initial public release
- Core build orchestration engine
- Cache-first incremental builds
- Multi-language backend support
- Basic CI/CD generation

[Unreleased]: https://github.com/foursavage-dev/forge-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/foursavage-dev/forge-rs/releases/tag/v0.1.0
