# AI Agent Workflow Guide for Fish

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

This document provides detailed step-by-step instructions for AI coding agents working with the fish build orchestration system. Following this workflow ensures minimal errors, smooth operation, and high-quality contributions.

## 🎯 Overview

This guide outlines a comprehensive workflow designed to:
- Minimize introduction of bugs and errors
- Ensure smooth project operation
- Maintain code quality and consistency
- Follow project-specific best practices
- Integrate properly with existing architecture

---

## 📖 Phase 1: Pre-Work Context Gathering

### Step 1.1: Read Essential Documentation (Mandatory)

**Order of reading:**
1. **README.md** - Project overview, quick start, basic commands
2. **Cargo.toml** - Workspace structure, dependencies, MSRV (1.88+)
3. **ARCHITECTURE.md** - System architecture, component responsibilities
4. **DEVELOPMENT.md** - Local development setup and workflow

**Why this order:**
- README provides project context and goals
- Cargo.toml reveals structure and technical constraints
- ARCHITECTURE.md explains how components interact
- DEVELOPMENT.md provides local environment setup

### Step 1.2: Read Task-Specific Documentation

**Based on your task, read additional files:**

| Task Type | Additional Files to Read |
|-----------|--------------------------|
| Language backend | `crates/fish-backend-rust/` (as example), `ARCHITECTURE.md` Backend section |
| Scheduler changes | `crates/fish-scheduler/` source files |
| Cache improvements | `crates/fish-cache/` and `crates/fish-cas/` source files |
| CLI modifications | `crates/fish-cli/` source files |
| Security features | `crates/fish-security/` and `crates/fish-signing/` |
| Distributed execution | `crates/fish-worker/` and `crates/fish-remote-cache/` |

### Step 1.3: Verify Current Project State

**Run these commands to understand current state:**
```bash
# Check git status
git status

# Check current branch
git branch

# View recent commits
git log --oneline -10

# Check if tests currently pass
cargo test --workspace
```

**Critical checks:**
- Ensure you're on `dev` branch (not `main`)
- Verify working directory is clean (no uncommitted changes)
- Confirm tests currently pass before making changes

---

## 🎯 Phase 2: Task Analysis and Planning

### Step 2.1: Understand the Task Requirements

**Ask these questions before coding:**
- What specific problem am I solving?
- Which crates/components are affected?
- Are there existing patterns I should follow?
- What are the potential side effects?

### Step 2.2: Identify Affected Components

**Map your task to specific crates:**
- Core functionality ➔ `fish-core`, `fish-graph`, `fish-executor`
- Scheduling logic ➔ `fish-scheduler`
- Caching ➔ `fish-cache`, `fish-cas`
- Language support ➔ `fish-backend-*`
- CLI ➔ `fish-cli`
- Security ➔ `fish-security`, `fish-signing`

### Step 2.3: Review Existing Patterns

**Before writing new code, examine similar existing code:**
- Look at how other backends implement the `Backend` trait
- Examine existing scheduler implementations for patterns
- Review cache implementations for consistency
- Check error handling patterns in similar code

### Step 2.4: Plan Changes

**Create a mental or written plan:**
1. Which files need to be modified?
2. What new files need to be created?
3. What tests need to be added/updated?
4. What documentation needs updating?
5. What are the potential risks?

---

## 🛠️ Phase 3: Implementation

### Step 3.1: Follow Rust Best Practices

**Code standards to follow:**
- Use `anyhow` for error handling in application code
- Use `thiserror` for custom error types in library code
- Follow Rust 2024 edition conventions
- Use meaningful variable and function names
- Keep functions focused and single-purpose
- Add rustdoc comments for public APIs

### Step 3.2: Maintain MSRV Compatibility

**Critical constraint:**
- **MSRV is Rust 1.88+**
- Do not use features requiring newer Rust versions
- Check `Cargo.toml` for `rust-version = "1.88"`
- Test with minimum Rust version if possible

### Step 3.3: Follow Project-Specific Patterns

**Use existing patterns from the codebase:**
- Error handling: Follow existing `Result` and `Error` type patterns
- Async patterns: Use `async-trait` where appropriate
- Testing: Follow existing test organization and naming
- Configuration: Use existing config patterns from `fish.toml`

### Step 3.4: Implement Incrementally

**Recommended approach:**
1. Make small, testable changes
2. Run tests after each significant change
3. Commit frequently with descriptive messages
4. Test integration points early

### Step 3.5: Add Tests

**Testing requirements:**
- Write unit tests for new functions
- Add integration tests for cross-crate functionality
- Test error paths and edge cases
- Ensure all tests pass: `cargo test --workspace`

### Step 3.6: Update Documentation

**Documentation updates:**
- Update rustdoc comments for changed APIs
- Update ARCHITECTURE.md for structural changes
- Update DEVELOPMENT.md for workflow changes
- Update relevant examples in `examples/`

---

## ✅¦ Phase 4: Verification and Validation

### Step 4.1: Run Full Test Suite

**Mandatory testing commands:**
```bash
# Run all workspace tests
cargo test --workspace

# Run tests with specific features if applicable
cargo test --workspace --all-features
```

**Test failure protocol:**
- If tests fail, investigate and fix before proceeding
- Ensure new tests cover your changes
- Verify existing tests still pass

### Step 4.2: Run Linting

**Mandatory linting commands:**
```bash
# Format check
cargo fmt --all -- --check

# Clippy with strict warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

**Lint fixing protocol:**
- Fix all clippy warnings
- Ensure code is properly formatted
- Address any compiler warnings

### Step 4.3: Build Verification

**Build commands:**
```bash
# Build the CLI
cargo build -p fish-cli

# Build all workspace members
cargo build --workspace

# Release build check
cargo build --workspace --release
```

### Step 4.4: Functional Testing

**Test the actual functionality:**
```bash
# Test basic Fish commands
cargo run -p fish-cli -- build
cargo run -p fish-cli -- --help

# Test specific functionality related to your changes
# (add task-specific commands here)
```

### Step 4.5: Integration Testing

**Test integration points:**
- If you modified a backend, test it with actual projects
- If you modified the scheduler, test with dependency graphs
- If you modified cache, test cache hit/miss scenarios
- If you modified CLI, test the specific commands

---

## 🔒 Phase 5: Security and Safety

### Step 5.1: Security Review

**Security considerations:**
- Review code for potential security vulnerabilities
- Check for unsafe code and ensure it's properly justified
- Verify no secrets or sensitive data are committed
- Check input validation and sanitization

### Step 5.2: Dependency Check

**Dependency verification:**
- Review new dependencies added to `Cargo.toml`
- Check for known vulnerabilities in dependencies
- Ensure dependencies are properly licensed
- Prefer dependencies that are actively maintained

### Step 5.3: Performance Considerations

**Performance impact assessment:**
- Consider performance implications of changes
- Avoid unnecessary allocations or copies
- Consider async vs sync appropriately
- Test performance if changes affect critical paths

---

## 📝 Phase 6: Documentation and Communication

### Step 6.1: Update CHANGELOG

**Changelog protocol:**
- Add entry to `CHANGELOG.md` following existing format
- Describe changes clearly and concisely
- Categorize changes (Added, Changed, Fixed, Removed)
- Reference relevant issues or PRs

### Step 6.2: Update Relevant Documentation

**Documentation checklist:**
- [ ] ARCHITECTURE.md (if structural changes)
- [ ] DEVELOPMENT.md (if workflow changes)
- [ ] README.md (if user-facing changes)
- [ ] Rustdoc comments (for API changes)
- [ ] Examples (if behavior changes)

### Step 6.3: Commit Guidelines

**Commit message format:**
```
[type]: brief description

Detailed explanation of the change, including:
- Why the change was made
- What problem it solves
- How it was implemented
- Any potential side effects

[type] can be: feat, fix, docs, style, refactor, test, chore
```

**Branch protocol:**
- All development on `dev` branch
- Never commit directly to `main`
- Use descriptive branch names for features

---

## 🚨 Phase 7: Troubleshooting Common Issues

### Issue 7.1: Build Failures

**Common causes and solutions:**
- **MSRV violation**: Check Rust version, adjust code for 1.88+ compatibility
- **Dependency conflicts**: Update dependencies in `Cargo.toml`
- **Missing features**: Add required features to cargo commands

### Issue 7.2: Test Failures

**Debugging protocol:**
1. Run individual test: `cargo test --package <crate> <test_name>`
2. Check test output for specific failure reasons
3. Review test setup and teardown
4. Verify test assumptions are still valid

### Issue 7.3: Linting Errors

**Clippy resolution:**
- Read clippy warnings carefully
- Understand why each warning exists
- Fix the underlying issue, not just suppress warnings
- Use `#[allow(clippy::...)]` only when truly necessary

### Issue 7.4: Integration Issues

**Cross-crate problems:**
- Verify public APIs are properly exported
- Check version consistency in workspace dependencies
- Ensure trait implementations are complete
- Verify type signatures match across crate boundaries

---

## 🎯 Phase 8: Quality Assurance Checklist

### Pre-Commit Checklist

**Before committing, verify:**
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Code is formatted: `cargo fmt --all -- --check`
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated
- [ ] MSRV compatibility maintained (Rust 1.88+)
- [ ] No secrets or sensitive data committed
- [ ] Changes are on `dev` branch
- [ ] Commit message follows guidelines

### Pre-PR Checklist

**Before creating pull request:**
- [ ] All pre-commit items verified
- [ ] Code reviewed against ARCHITECTURE.md patterns
- [ ] Integration tests performed
- [ ] Performance impact assessed
- [ ] Security review completed
- [ ] Documentation is comprehensive
- [ ] Tests cover new functionality
- [ ] CHANGELOG entry added

---

## 🔄 Phase 9: Continuous Improvement

### Step 9.1: Learn from Issues

**When issues arise:**
- Document the root cause
- Update this workflow if needed
- Share lessons learned
- Improve testing to prevent recurrence

### Step 9.2: Update This Guide

**Guide maintenance:**
- Update this document when workflows change
- Add new patterns as they emerge
- Remove outdated practices
- Keep examples current

---

## 📊 Quick Reference Commands

### Essential Commands
```bash
# Build
cargo build -p fish-cli
cargo build --workspace

# Test
cargo test --workspace
cargo test --workspace --all-features

# Lint
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run Fish
cargo run -p fish-cli -- build
cargo run -p fish-cli -- test
cargo run -p fish-cli -- --help
```

### Verification Commands
```bash
# Check Rust version
rustc --version

# Check workspace status
cargo check --workspace

# List workspace members
cargo tree --workspace

# Check for security vulnerabilities
cargo audit
```

---

## 🏆 Best Practices Summary

### Do's
- ✅¦ Read documentation before coding
- ✅¦ Follow existing patterns in the codebase
- ✅¦ Write tests for new functionality
- ✅¦ Run full test suite before committing
- ✅¦ Fix all clippy warnings
- ✅¦ Update documentation
- ✅¦ Use meaningful commit messages
- ✅¦ Develop on `dev` branch
- ✅¦ Consider performance implications
- ✅¦ Review security implications

### Don'ts
- ❌ Skip reading ARCHITECTURE.md
- ❌ Commit directly to `main` branch
- ❌ Ignore test failures
- ❌ Suppress clippy warnings without justification
- ❌ Use unstable features without MSRV consideration
- ❌ Commit secrets or sensitive data
- ❌ Make large changes without testing
- ❌ Break existing APIs without documentation
- ❌ Ignore error handling patterns
- ❌ Skip integration testing

---

## 🚨 Emergency Procedures

### When Something Goes Wrong

**1. Build fails:**
```bash
# Clean build
cargo clean
cargo build --workspace
```

**2. Tests fail unexpectedly:**
```bash
# Update dependencies
cargo update
cargo test --workspace
```

**3. Clippy errors:**
```bash
# Check specific clippy lints
cargo clippy --workspace -- -W clippy::<specific_lint>
```

**4. Git issues:**
```bash
# Reset to clean state
git status
git checkout -- .
git clean -fd
```

---

## 📞 Additional Resources

### Project Documentation
- [README.md](getting-started.md) - Project overview
- [ARCHITECTURE.md](architecture.md) - System architecture
- [DEVELOPMENT.md](development.md) - Development setup
- [CONTRIBUTING.md](contributing.md) - Contribution guidelines
- [ROADMAP.md](ROADMAP.md) - Project roadmap

### External Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

---

## 🎯 Success Criteria

**A successful AI agent contribution should:**
- ✅¦ Pass all workspace tests
- ✅¦ Have zero clippy warnings
- ✅¦ Be properly formatted
- ✅¦ Follow project patterns
- ✅¦ Include comprehensive tests
- ✅¦ Update relevant documentation
- ✅¦ Maintain MSRV compatibility
- ✅¦ Be properly committed to `dev` branch
- ✅¦ Include meaningful commit messages
- ✅¦ Not introduce security vulnerabilities

---

**Following this workflow ensures that AI agents can contribute to Fish effectively while maintaining code quality, minimizing errors, and ensuring smooth project operation.**