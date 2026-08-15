# Rust Backend

The Rust backend provides build orchestration for Rust projects using Cargo.

## Detection

The Rust backend is detected when a `Cargo.toml` file is present in the project directory.

## Configuration

### forge.rust.json

```json
{
  "features": ["default"],
  "all-features": false,
  "doc": false,
  "benches": false,
  "examples": false,
  "tests": true
}
```

### Configuration Options

- `features`: Default features to enable
- `all-features`: Whether to enable all features
- `doc`: Whether to build documentation
- `benches`: Whether to run benchmarks
- `examples`: Whether to build examples
- `tests`: Whether to run tests

## Tasks Generated

### Build Task

```bash
cargo build --release --features <features>
```

### Test Task

```bash
cargo test --release --features <features>
```

### Check Task

```bash
cargo check --release --features <features>
```

### Doc Task

```bash
cargo doc --release --features <features>
```

## Dependency Extraction

The Rust backend extracts dependencies from:

- `Cargo.toml` dependencies section
- `Cargo.lock` for exact versions
- Workspace dependencies

## Fingerprinting

The Rust backend fingerprints:

- `Cargo.toml` content
- `Cargo.lock` content
- Source files (excluding target/)
- Build configuration

## Examples

### Basic Rust Project

```bash
cd my-rust-project
forge build
```

### Workspace with Features

```bash
cd my-workspace
forge build -p my-package --features "serde,uuid"
```

### Workspace with Tests

```bash
cd my-workspace
forge test
```

## Limitations

- Requires Rust toolchain installed
- Cargo workspaces supported
- Custom build scripts supported via procedural macros

## Performance Optimization

The Rust backend uses:

- **Level batching**: Groups independent packages
- **Feature-aware caching**: Cache per feature combination
- **Workspace-aware**: Optimizes for workspace builds

## Troubleshooting

### Cargo not found

Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Build fails with linking errors

Check that native dependencies are installed and toolchain is correct.

### Cache not working

Clear cache: `forge cache prune` and rebuild.
