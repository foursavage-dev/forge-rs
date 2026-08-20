# Release Process

> ðŸŒ **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

This document describes the process for releasing new versions of Fish.

## Versioning

Fish follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html):

- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible functionality additions
- **PATCH**: Backwards-compatible bug fixes

## Pre-Release Checklist

### Quality Assurance

- [ ] All tests pass on all platforms (Linux, macOS, Windows)
- [ ] Clippy passes with no warnings: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Code is formatted: `cargo fmt --all -- --check`
- [ ] Documentation builds without warnings: `cargo doc --workspace --no-deps`
- [ ] No unresolved `TODO` or `FIXME` comments in production code
- [ ] Security audit completed (for major/minor releases)

### Documentation

- [ ] CHANGELOG.md updated with all changes
- [ ] README.md updated if features changed
- [ ] ARCHITECTURE.md updated if architecture changed
- [ ] API documentation updated
- [ ] Migration guide if breaking changes

### Release Notes

- [ ] Release notes written
- [ ] Breaking changes documented
- [ ] New features highlighted
- [ ] Bug fixes listed
- [ ] Upgrade instructions provided

## Release Process

### 1. Version Bump

Update version in workspace `Cargo.toml`:

```toml
[workspace.package]
version = "0.2.0"  # Bump version
```

Update version in all crate `Cargo.toml` files if they have explicit versions.

### 2. Update CHANGELOG

Add new section to CHANGELOG.md:

```markdown
## [0.2.0] - 2026-XX-XX

### Added
- New feature descriptions

### Changed
- Breaking changes

### Fixed
- Bug fixes

### Security
- Security fixes
```

### 3. Create Release Branch

```bash
git checkout -b release/v0.2.0
```

### 4. Final Testing

```bash
# Run full test suite
cargo test --workspace

# Run clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check formatting
cargo fmt --all -- --check

# Build release
cargo build --release
```

### 5. Commit Changes

```bash
git add .
git commit -m "chore: release v0.2.0"
```

### 6. Tag Release

```bash
git tag -a v0.2.0 -m "Release v0.2.0"
```

### 7. Push to GitHub

```bash
git push upstream release/v0.2.0
git push upstream v0.2.0
```

### 8. Create GitHub Release

1. Go to GitHub releases page
2. Click "Draft a new release"
3. Select tag `v0.2.0`
4. Add release title and description
5. Attach binaries if applicable
6. Publish release

### 9. Publish to crates.io

```bash
# Publish all crates
cargo publish --workspace

# Or publish individual crates
cargo publish -p fish-core
cargo publish -p fish-cli
# ... continue for all crates
```

### 10. Update Main Branch

```bash
git checkout main
git merge release/v0.2.0
git push upstream main
```

### 11. Announce Release

- Update Discord community
- Post on social media
- Send release notes to mailing list
- Update website if applicable

## Post-Release

### Cleanup

- [ ] Delete release branch after merge
- [ ] Close related issues
- [ ] Update roadmap
- [ ] Plan next release

### Monitoring

- Monitor for bug reports
- Track download metrics
- Gather user feedback
- Update documentation based on feedback

## Emergency Releases

For critical security issues or severe bugs:

1. Skip normal release process
2. Create hotfix branch from release tag
3. Apply fix
4. Create patch version (0.1.1 -> 0.1.2)
5. Follow release process
6. Announce security update

## Release Schedule

- **Major releases**: As needed, significant features
- **Minor releases**: Monthly or when significant features are ready
- **Patch releases**: As needed for bug fixes

## Release Artifacts

### Binary Distribution

```bash
# Build release binaries
cargo build --release

# Package binaries
# Linux
target/release/Fish -> fish-linux-x86_64

# macOS
target/release/Fish -> fish-macos-x86_64
target/release/Fish -> fish-macos-arm64

# Windows
target/release/Fish.exe -> fish-windows-x86_64.exe
```

### Docker Images

```bash
# Build Docker image
docker build -t foursavage/Fish:0.2.0 .

# Push to registry
docker push foursavage/Fish:0.2.0
docker push foursavage/Fish:latest
```

## Verification

After release, verify:

- [ ] Download artifacts from GitHub release
- [ ] Test installation from artifacts
- [ ] Verify version: `Fish --version`
- [ ] Run basic build test
- [ ] Check documentation links
- [ ] Verify crates.io publication

## Rollback Plan

If a release has critical issues:

1. Delete GitHub release
2. Yank crates.io packages
3. Update documentation
4. Communicate issue to users
5. Prepare hotfix release

## Release Communication

### Release Notes Template

```markdown
## Fish v0.2.0 Release

### Highlights
- Major feature 1
- Major feature 2

### New Features
- Feature description
- Feature description

### Breaking Changes
- Breaking change description with migration guide

### Bug Fixes
- Bug fix description

### Upgrading
Instructions for upgrading from previous version

### Thanks
Thanks to contributors for this release
```

## Support Period

- **Current version**: Full support
- **Previous version**: Security patches only
- **Older versions**: No support

Typically support the last 2 minor versions.
