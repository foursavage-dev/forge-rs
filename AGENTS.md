# AI Agent Guide for Fish

This document provides guidance for AI coding agents working with the fish build orchestration system.

## 🎯 Purpose

Fish is a Rust build orchestration experiment for projects that use more than one toolchain. This guide helps AI agents understand the project structure, workflow, and best practices to ensure smooth operation and high-quality contributions.

## 📖 Context Reading Order

AI agents should read files in this priority order:

### Phase 1: Initial Context (Must Read)
1. **README.md** - Project overview, quick start, basic commands
2. **Cargo.toml** - Workspace structure, dependencies, MSRV (1.88+)
3. **ARCHITECTURE.md** - System architecture, component responsibilities

### Phase 2: Development Context (Read Before Editing)
4. **DEVELOPMENT.md** - Local setup, testing, build instructions
5. **CONTRIBUTING.md** - Contribution guidelines, code standards

### Phase 3: Detailed Workflow (Read Before Starting Work)
6. **docs/AI_AGENT_WORKFLOW.md** - **Comprehensive step-by-step workflow guide**

## 🔑 Critical Information

### Version Requirements
- **MSRV**: Rust 1.88+ (Minimum Supported Rust Version)
- **Edition**: Rust 2024
- **Workspace**: 36+ crates with resolver = "2"

### Branch Policy
- **`dev`** branch - All development happens here
- **`main`** branch - Stable code only, merge from `dev`
- Never commit directly to `main` during development

### Testing Requirements
```bash
# Always run before committing
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## 🏗️ Architecture Overview

### Core Components
- **fish-core**: Workspace discovery, manifest model, DAG merger
- **fish-graph**: Dependency graph, topological sort, query algebra
- **fish-executor**: Process execution, middleware chain, response files
- **fish-scheduler**: Parallel scheduling, jobserver pool, racing, DTE
- **fish-cache**: Fingerprint cache and two-phase pruning
- **fish-cas**: Content-addressable artifact storage with ZSTD compression

### Language Backends (11+)
Rust, C/C++, Go, TypeScript/JavaScript, Python, Java, .NET, Swift, Dart, Zig, Docker

### Advanced Features
- Remote cache/worker, sandbox, signing, security scanning
- Web dashboard with DAG visualization
- File watching, affected-project detection
- Profile-guided optimization (PGO)

## 🛠️ Common Tasks

### Adding a New Language Backend
1. Create crate: `crates/fish-backend-{lang}/`
2. Implement `Backend` trait:
   ```rust
   pub trait Backend {
       fn detect(&self, path: &Path) -> bool;
       fn extract_dependencies(&self, path: &Path) -> Result<Vec<Dependency>>;
       fn generate_tasks(&self, package: &Package) -> Result<Vec<Task>>;
   }
   ```
3. Add to `Cargo.toml` workspace members
4. Register in `fish-core/src/backend/mod.rs`

### Modifying Scheduler Logic
- **Primary crate**: `crates/fish-scheduler/`
- **Key files**: 
  - `src/scheduler.rs` - Main scheduling logic
  - `src/jobserver.rs` - GNU Jobserver integration
  - `src/racing.rs` - Dynamic remote racing
- **Consider**: Concurrency limits, resource governance, task dependencies

### Cache Improvements
- **Fingerprint cache**: `crates/fish-cache/src/`
- **CAS storage**: `crates/fish-cas/src/`
- **Key considerations**: Blake3 hashing, ZSTD compression, tiered L1/L2 caching

### CLI Changes
- **Location**: `crates/fish-cli/src/`
- **Key files**: `main.rs`, command modules
- **Consider**: Error handling, user experience, help text

## 📝 Code Standards

### Rust Patterns
- Use `anyhow` for error handling
- Use `thiserror` for custom error types
- Prefer async/await where appropriate
- Follow Rust 2024 edition conventions

### Testing
- Write unit tests for new features
- Ensure workspace tests pass: `cargo test --workspace`
- Consider integration tests for cross-crate functionality

### Documentation
- Document public APIs with rustdoc comments
- Update ARCHITECTURE.md for structural changes
- Update DEVELOPMENT.md for workflow changes

## 🚨 Before Committing

1. **Test**: `cargo test --workspace`
2. **Lint**: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. **Format**: `cargo fmt --all -- --check`
4. **Review**: Check CONTRIBUTING.md guidelines
5. **Branch**: Ensure changes are on `dev` branch

## 🎯 Comprehensive Workflow Guide

**For detailed step-by-step instructions on how to work with this project, including:**
- Pre-work context gathering
- Task analysis and planning
- Implementation best practices
- Verification and validation procedures
- Security and safety considerations
- Quality assurance checklists
- Troubleshooting common issues

**Please read:** [docs/AI_AGENT_WORKFLOW.md](docs/AI_AGENT_WORKFLOW.md)

## 🎯 Quick Reference

### File Locations
- Core logic: `crates/fish-core/`, `crates/fish-graph/`, `crates/fish-executor/`
- Scheduling: `crates/fish-scheduler/`
- Caching: `crates/fish-cache/`, `crates/fish-cas/`
- CLI: `crates/fish-cli/`
- Backends: `crates/fish-backend-*/`

### Key Commands
```bash
# Build
cargo build -p fish-cli

# Test
cargo test --workspace

# Development
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Dependencies
- Core: anyhow, serde, serde_json, toml, clap, regex, blake3
- Async: async-trait, tokio (implied)
- UI: ratatui, crossterm, anstyle

## 📞 Additional Resources

### Project Documentation
- [README.md](README.md) - Project overview
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [DEVELOPMENT.md](DEVELOPMENT.md) - Development setup
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [ROADMAP.md](ROADMAP.md) - Project roadmap
- [docs/AI_AGENT_WORKFLOW.md](docs/AI_AGENT_WORKFLOW.md) - Detailed workflow guide

### External Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
