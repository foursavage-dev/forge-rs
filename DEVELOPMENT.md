# Development Guide

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

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
git clone https://github.com/requla11/fish.git
cd fish

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
├── crates/                 # All workspace crates
│   ├── fish-core/         # Core functionality
│   ├── fish-cli/          # CLI interface
│   ├── fish-backend-*/    # Language backends
│   └── ...                 # Other crates
├── examples/               # Example projects
├── docs/                   # Documentation
├── tests/                  # Integration tests
├── Cargo.toml              # Workspace configuration
└── README.md               # Project overview
```

## Building

### Development Build

```bash
# Build the CLI
cargo build -p fish-cli

# Build with debug output
cargo build -p fish-cli --verbose

# Build release version
cargo build -p fish-cli --release
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test -p fish-core test_name

# Run tests with output
cargo test --workspace -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Testing

### Unit Tests

Unit tests are located within each crate's `src/` directory.

```bash
# Run unit tests for a specific crate
cargo test -p fish-core
```

### Integration Tests

Integration tests are located in the `tests/` directory.

```bash
# Run integration tests
cargo test --test integration
```

### Testing Backends

To test specific language backends:

```bash
# Test Rust backend
cargo test -p fish-backend-rust

# Test Go backend
cargo test -p fish-backend-go
```

## Debugging

### Common Issues

**Build fails with MSRV error:**
- Ensure you have Rust 1.88 or later installed
- Use `rustup update` to update Rust

**Tests fail sporadically:**
- Run tests with `cargo test --workspace -- --test-threads=1`
- Check for race conditions or filesystem timing issues

**Cache issues:**
- Clear cache: `rm -rf ~/.fish/cache`
- Disable cache temporarily: `fish build --no-cache`

### Debug Logging

Enable debug logging:

```bash
RUST_LOG=debug fish build
RUST_LOG=debug fish build --verbose
```

## Performance Profiling

### Build Profiling

```bash
# Profile build time
fish build --profile build-trace.json

# Analyze profile
cargo install flamegraph
cargo flamegraph --bin fish
```

### Memory Profiling

```bash
# Run with memory profiling
valgrind --leak-check=full fish build
```

## Documentation

### Building Documentation

```bash
# Build documentation
cargo doc --workspace --no-deps

# Open documentation in browser
cargo doc --workspace --open
```

### Documentation Standards

- Document public APIs with rustdoc comments
- Include examples in documentation
- Document error conditions and edge cases
- Keep documentation up-to-date with code changes

## Commit Guidelines

### Commit Message Format

Follow conventional commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Example Commit

```
feat(scheduler): add dynamic task scheduling

Implement dynamic task scheduling based on resource availability
and task dependencies. This improves build parallelism for large
workspaces.

Closes #123
```

## Pull Request Process

1. Create a new branch from `dev`
2. Make your changes
3. Run tests and ensure they pass
4. Update documentation if needed
5. Submit a pull request to `dev`
6. Address review feedback
7. Once approved, maintainers will merge to `dev`

## Getting Help

- Check existing issues on GitHub
- Read ARCHITECTURE.md for system design questions
- Join the community discussions
- Contact maintainers for security issues

## Release Process

Releases are handled by maintainers following these steps:

1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Create git tag
4. Build release artifacts
5. Publish to crates.io
6. Create GitHub release

See RELEASING.md for detailed release procedures.
