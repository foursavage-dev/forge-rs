# Support

> ðŸŒ **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

This document provides support resources for Fish users.

## Getting Help

### Documentation

- [README.md](README.md) - Project overview and quick start
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [DEVELOPMENT.md](DEVELOPMENT.md) - Development guide
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [ROADMAP.md](ROADMAP.md) - Project roadmap

### Community

- **Discord**: Join our Discord community for real-time help
- **GitHub Issues**: Report bugs and request features
- **GitHub Discussions**: Ask questions and share ideas
- **Email**: foursavage@proton.me

## Common Issues

### Installation Problems

#### Fish not found after installation

**Solution**: Ensure the installation directory is in your PATH:

```bash
# Linux/macOS
export PATH="$HOME/.local/bin:$PATH"

# Windows
# Add Fish installation directory to PATH
```

#### Permission denied on Linux/macOS

**Solution**: Make the binary executable:

```bash
chmod +x ~/.local/bin/Fish
```

### Build Issues

#### Build fails with "command not found"

**Solution**: Install required toolchains:

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (for TypeScript backend)
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt-get install -y nodejs

# Python (for Python backend)
pip install uv

# Go (for Go backend)
go install golang.org/dl/go@latest
```

#### Cache errors

**Solution**: Clear the cache:

```bash
Fish cache prune
```

### Performance Issues

#### Slow builds

**Solutions**:
1. Check cache hit rate: `Fish cache stats`
2. Enable parallel builds: `Fish build -j $(nproc)`
3. Use distributed workers if available
4. Profile with: `Fish build --profile`

#### High memory usage

**Solutions**:
1. Reduce parallel jobs: `Fish build -j 4`
2. Clear cache: `Fish cache prune`
3. Check for memory leaks

### CI/CD Issues

#### CI generation fails

**Solution**: Check CI platform support:

```bash
# Supported platforms
Fish ci init --platform github    # GitHub Actions
Fish ci init --platform gitlab    # GitLab CI
Fish ci init --platform circleci  # CircleCI
Fish ci init --platform bitbucket # Bitbucket Pipelines
```

#### Generated CI doesn't work

**Solution**: 
1. Check platform requirements
2. Verify toolchain installation in CI
3. Review generated configuration
4. Report issue with details

### Security Issues

#### Vulnerability scan fails

**Solution**: Update dependencies:

```bash
# Update Rust dependencies
cargo update

# Update Node dependencies
npm update

# Update Python dependencies
pip install --upgrade pip
```

#### Artifact verification fails

**Solution**: 
1. Verify signing key is trusted
2. Check signature format
3. Ensure artifact wasn't modified
4. Report if verification incorrectly fails

### Distributed Build Issues

#### Worker connection fails

**Solution**:
1. Check network connectivity
2. Verify worker is running
3. Check authentication
4. Review firewall settings

#### VFS file access fails

**Solution**:
1. Ensure VFS is enabled
2. Check file permissions
3. Verify mount configuration
4. Check disk space

## Troubleshooting Steps

### General Troubleshooting

1. **Check version**: `Fish --version`
2. **Check environment**: `Fish doctor`
3. **Enable debug logging**: `RUST_LOG=debug Fish build`
4. **Clear cache**: `Fish cache prune`
5. **Update Fish**: Install latest version
6. **Check for known issues**: GitHub issues

### Diagnostic Information

When reporting issues, include:

```bash
# Version
Fish --version

# Environment
Fish doctor

# System info
uname -a

# Rust version
rustc --version

# Detailed logs
RUST_LOG=debug Fish build 2>&1 | tee debug.log
```

## Feature Requests

### Before Requesting

1. Check if feature already exists
2. Search existing feature requests
3. Review roadmap
4. Consider contributing the feature

### Submitting Feature Requests

Use the [Feature Request Template](.github/ISSUE_TEMPLATE/feature_request.md):

```markdown
**Is your feature request related to a problem?**
Describe the problem

**Describe the solution**
What you want to happen

**Alternatives considered**
Other solutions you thought of

**Additional context**
Any other information
```

## Bug Reports

### Before Reporting

1. Check if bug is already reported
2. Try to reproduce the issue
3. Check if you're using the latest version
4. Try with a clean environment

### Submitting Bug Reports

Use the [Bug Report Template](.github/ISSUE_TEMPLATE/bug_report.md):

```markdown
**Description**
What happened

**To Reproduce**
Steps to reproduce

**Expected Behavior**
What should happen

**Environment**
OS, Rust version, Fish version

**Additional Context**
Logs, screenshots, etc.
```

## Contributing

Want to contribute? See [CONTRIBUTING.md](CONTRIBUTING.md).

## Professional Support

For enterprise support, SLAs, and priority assistance:
- **Email**: foursavage@proton.me
- **Website**: https://foursavage.dev

## Resources

### Learning Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Cargo Guide](https://doc.rust-lang.org/cargo/)

### Community Resources

- [Rust Discord](https://discord.gg/rust-lang)
- [Rust Reddit](https://www.reddit.com/r/rust/)
- [Rust Forums](https://users.rust-lang.org/)

## Acknowledgments

Support from the community helps make Fish better. Thank you!
