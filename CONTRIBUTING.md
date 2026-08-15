# Contributing to Forge

Thank you for your interest in contributing to Forge! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style and Guidelines](#code-style-and-guidelines)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Enhancements](#suggesting-enhancements)
- [Code Review Process](#code-review-process)

## Getting Started

### Prerequisites

- Rust 1.85 or later
- Git
- Basic familiarity with Rust and the command line

### Setting Up Development Environment

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/forge-rs.git
   cd forge-rs
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/foursavage-dev/forge-rs.git
   ```

## Development Setup

### Building the Project

```bash
# Build the entire workspace
cargo build --workspace

# Build a specific crate
cargo build -p forge-cli

# Build with optimizations
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p forge-cli

# Run tests with output
cargo test --workspace -- --nocapture

# Run integration tests
cargo test --workspace --test-threads=1
```

### Code Quality Checks

```bash
# Run Clippy linter
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

## Code Style and Guidelines

### Rust Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `#![forbid(unsafe_code)]` in security-sensitive crates
- Prefer safe abstractions over unsafe code
- Document public APIs with `///` doc comments
- Use `Result` types for error handling
- Avoid `unwrap()` and `expect()` in production code

### Project-Specific Guidelines

- Use `thiserror` for error types
- Use `anyhow` for application errors when appropriate
- Follow the existing crate structure and module organization
- Keep crates focused on their specific responsibility
- Add tests for new functionality
- Update documentation when adding features

### Workspace Structure

```
forge-rs/
├── crates/
│   ├── forge-core/           # Core functionality
│   ├── forge-cli/            # Command-line interface
│   ├── forge-backend-*/      # Language backends
│   └── ...                   # Other feature crates
├── examples/                 # Example projects
└── docs/                     # Documentation
```

## Testing

### Writing Tests

- Write unit tests alongside the code they test
- Add integration tests in the `tests/` directory
- Use descriptive test names
- Test both success and failure cases

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Test implementation
    }

    #[tokio::test]
    async fn test_async_feature() {
        // Async test implementation
    }
}
```

## Submitting Changes

### Commit Message Format

Follow conventional commits format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`

Example:
```
feat(cli): add new build command

Add a new command to build specific packages with custom options.

Closes #123
```

### Pull Request Process

1. Create a new branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. Make your changes and commit them
3. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```
4. Create a pull request on GitHub
5. Fill out the PR template
6. Wait for code review
7. Address review feedback
8. Once approved, maintainers will merge

### PR Requirements

- All tests must pass
- Code must be formatted with `cargo fmt`
- Clippy must pass with no warnings
- Documentation must be updated if needed
- Commits must follow the commit message format

## Reporting Bugs

### Before Reporting

1. Check existing issues to avoid duplicates
2. Search the documentation
3. Try to reproduce the issue

### Bug Report Template

```markdown
**Description**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected Behavior**
A clear description of what you expected to happen.

**Environment**
- OS: [e.g. Windows 10, macOS 12.0]
- Rust version: [e.g. 1.85.0]
- Forge version: [e.g. 0.1.0]

**Additional Context**
Add any other context about the problem here.
```

## Suggesting Enhancements

### Feature Request Template

```markdown
**Is your feature request related to a problem?**
A clear description of what the problem is.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
A clear description of any alternative solutions or features you've considered.

**Additional context**
Add any other context or screenshots about the feature request here.
```

## Code Review Process

### Review Guidelines

- Be constructive and respectful
- Focus on the code, not the person
- Provide clear, actionable feedback
- Ask questions if something is unclear
- Consider the broader impact of changes

### Timeline

- Initial review: 1-3 business days
- Follow-up review: 1-2 business days after changes
- Merge: 1-2 business days after approval

## Getting Help

- Open an issue for bugs or feature requests
- Join our Discord community (link coming soon)
- Email: foursavage@proton.me
- Check the [documentation](docs/)

## License

By contributing to Forge, you agree that your contributions will be licensed under the MIT License.
