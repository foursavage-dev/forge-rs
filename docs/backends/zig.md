# Zig Backend

Fish delivers zero-overhead coordination for Zig build scripts and C/C++ cross-compilation toolchains.

## Detection
Fish automatically identifies Zig projects by locating:
- `build.zig`
- `build.zig.zon`

## Supported Commands
```bash
fish build     # Executes zig build
fish test      # Executes zig build test
fish check     # Validates Zig syntax and AST
```

## Configuration in `fish.toml`
```toml
backend = "zig"
jobs = 8
```
