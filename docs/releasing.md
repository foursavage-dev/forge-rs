# Release Guide

This document outlines the standard release lifecycle and checklist for Fish.

## Release Checklist
1. **Pre-flight Quality Verification**:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   ```
2. **Version Bump**:
   Update `Cargo.toml`, `Cargo.lock`, and `vscode-extension/package.json`.
3. **Changelog Update**:
   Document all new features, fixes, and breaking changes in `CHANGELOG.md`.
4. **Git Tagging**:
   ```bash
   git tag -a v0.3.0 -m "Release v0.3.0"
   git push origin v0.3.0
   ```
5. **Artifact Publishing**:
   - Publish crates to crates.io
   - Package VS Code `.vsix`
   - Release binaries on GitHub Releases
