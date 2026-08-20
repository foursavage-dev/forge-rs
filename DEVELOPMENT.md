# Development Guide

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

This guide provides detailed information for developers working on Fish.

## Development Environment

### Prerequisites

- Rust 1.88 or later (MSRV 1.88)
- Git
- A text editor or IDE (VS Code recommended)
- Docker (for testing Docker backend)

### Setting Up

```bash
# Clone the repository
git clone https://github.com/foursavage-dev/fish-rs.git
cd fish-rs

# Install development dependencies
cargo install cargo-watch
cargo install cargo-edit
cargo install cargo-expand
```

### VS Code Setup

Install the recommended extensions:
- rust-analyzer
- CodeLLDB
- Even Better TOML
- Error Lens

## Workspace Structure

```
fish-rs/
â”œâ”€â”€ crates/                 # All workspace crates
â”‚   â”œâ”€â”€ fish-core/         # Core functionality
â”‚   â”œâ”€â”€ fish-cli/          # CLI interface
â”‚   â”œâ”€â”€ fish-backend-*/    # Language backends
â”‚   â””â”€â”€ ...                 # Other crates
â”œâ”€â”€ examples/               # Example projects
â”œâ”€â”€ docs/                   # Documentation
â”œâ”€â”€ tests/                  # Integration tests
â”œâ”€â”€ Cargo.toml              # Workspace configuration
â””â”€â”€ README.md               # Project overview
```

## Building

### Development Build

```bash
# Build entire workspace
cargo build --workspace

# Build specific crate
cargo build -p fish-cli

# Build with all features
cargo build --workspace --all-features
```

### Release Build

```bash
# Build release version
cargo build --release

# Build specific crate in release mode
cargo build -p fish-cli --release
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p fish-cli

# Run tests with output
cargo test --workspace -- --nocapture

# Run integration tests only
cargo test --workspace --test-threads=1 --test '*_test.rs'
```

### Test Organization

- Unit tests: Inside each crate's `src/` directory
- Integration tests: In `tests/` directory at workspace root
- Backend tests: In each backend crate's `tests/` directory

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Test implementation
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_functionality() {
        // Async test implementation
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

## Code Quality

### Clippy

```bash
# Run Clippy on workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run Clippy on specific crate
cargo clippy -p fish-cli -- -D warnings
```

### Formatting

```bash
# Format all code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

### Documentation

```bash
# Build documentation
cargo doc --workspace --no-deps

# Open documentation in browser
cargo doc --workspace --no-deps --open
```

## Debugging

### Debugging CLI

```bash
# Run CLI with debug output
RUST_LOG=debug cargo run --bin Fish -- build

# Run with specific log level
RUST_LOG=fish_core=debug cargo run --bin Fish -- build
```

### Debugging Tests

```bash
# Run specific test with output
cargo test -p fish-cli test_name -- --nocapture

# Run test with debugger
rust-gdb target/debug/fish-cli test_name
```

## Profiling

### CPU Profiling

```bash
# Build with profiling support
cargo build --release

# Run with perf (Linux)
perf record -g target/release/Fish build
perf report

# Use flamegraph (Linux)
cargo install flamegraph
cargo flamegraph --bin Fish build
```

### Memory Profiling

```bash
# Use heaptrack (Linux)
heaptrack target/release/Fish build

# Use valgrind (Linux)
valgrind --leak-check=full target/release/Fish build
```

## Working with Backends

### Adding a New Backend

1. Create new crate: `crates/fish-backend-<lang>/`
2. Implement the backend trait
3. Add to workspace members in `Cargo.toml`
4. Add backend detection in `fish-core`
5. Write tests
6. Add documentation

### Backend Implementation Pattern

```rust
pub struct Backend;

impl fish_core::backend::Backend for Backend {
    fn detect(&self, path: &Path) -> bool {
        // Detect if project uses this backend
    }

    fn extract_dependencies(&self, path: &Path) -> Result<Vec<Dependency>> {
        // Extract dependencies from project
    }

    fn generate_tasks(&self, package: &Package) -> Result<Vec<Task>> {
        // Generate build tasks
    }
}
```

## Working with Plugins

### Plugin Development

1. Create plugin directory: `.Fish/plugins/<plugin-name>/`
2. Create `plugin.json` configuration
3. Implement plugin script
4. Test with `Fish plugin execute <name> <command>`

### Plugin Configuration

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "My custom plugin",
  "type": "shell",
  "main": "plugin.sh",
  "commands": {
    "build": "plugin.sh build",
    "test": "plugin.sh test"
  }
}
```

## CI/CD Development

### Testing CI Generation

```bash
# Generate GitHub Actions workflow
cargo run --bin Fish -- ci init --platform github

# Generate GitLab CI pipeline
cargo run --bin Fish -- ci init --platform gitlab

# Generate CircleCI config
cargo run --bin Fish -- ci init --platform circleci

# Generate Bitbucket Pipelines
cargo run --bin Fish -- ci init --platform bitbucket
```

### Adding New CI Platform

1. Add platform to `CIPlatform` enum in `fish-ci-generator`
2. Create generator module in `crates/fish-ci-generator/src/`
3. Implement generator trait
4. Add CLI parsing
5. Write tests
6. Update documentation

## Security Development

### Testing Security Features

```bash
# Test artifact signing
cargo run --bin Fish -- sign artifact.sig

# Test vulnerability scanning
cargo run --bin Fish -- security scan

# Test secret management
cargo run --bin Fish -- secrets get my-secret
```

### Security Guidelines

- Never commit secrets or keys
- Use environment variables for secrets
- Audit all code with security tools
- Follow security best practices
- Report vulnerabilities responsibly

## Performance Optimization

### Performance Testing

```bash
# Run benchmarks
cargo bench --workspace

# Profile specific operation
cargo flamegraph --bin Fish build

# Analyze cache performance
cargo run --bin Fish -- cache stats
```

### Optimization Strategies

1. Profile to identify hotspots
2. Optimize critical paths
3. Reduce allocations
4. Use efficient data structures
5. Parallelize where possible
6. Cache expensive operations

## Documentation

### Writing Documentation

- Use `///` for public API documentation
- Include examples for complex APIs
- Add module-level documentation
- Keep documentation up to date
- Document behavior in edge cases

### Documentation Examples

```rust
/// Computes the fingerprint of a file.
///
/// # Arguments
///
/// * `path` - Path to the file to fingerprint
///
/// # Returns
///
/// Returns the BLAKE3 hash of the file content.
///
/// # Examples
///
/// ```
/// use fish_core::fingerprint;
///
/// let hash = fingerprint::compute_file("Cargo.toml").unwrap();
/// println!("Hash: {}", hash);
/// ```
pub fn compute_file(path: &Path) -> Result<String> {
    // Implementation
}
```

## Release Process

### Pre-Release Checklist

- [ ] All tests pass
- [ ] Clippy passes with no warnings
- [ ] Documentation is updated
- [ ] CHANGELOG is updated
- [ ] Version is bumped
- [ ] Release notes are written

### Release Steps

1. Update version in `Cargo.toml`
2. Update CHANGELOG
3. Create release branch
4. Run full test suite
5. Tag release
6. Push to GitHub
7. Create GitHub release
8. Publish to crates.io

## Troubleshooting

### Common Issues

**Build fails with "command not found"**
- Ensure all required tools are installed
- Check PATH environment variable
- Verify toolchain installation

**Tests fail intermittently**
- Check for race conditions
- Ensure tests are isolated
- Use proper test setup/teardown

**Memory usage high**
- Check for memory leaks
- Review caching strategy
- Profile with memory tools

### Getting Help

- Check existing issues
- Read documentation
- Ask in Discord community
- Contact maintainers

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed contribution guidelines.

## Code Review

### Review Checklist

- [ ] Code follows style guidelines
- [ ] Tests are included
- [ ] Documentation is updated
- [ ] No security issues
- [ ] Performance impact considered
- [ ] Backward compatibility maintained

### Review Process

1. Self-review your changes
2. Request review from maintainers
3. Address feedback
4. Update tests and documentation
5. Get final approval
6. Merge changes
