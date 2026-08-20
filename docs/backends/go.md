# Go Backend Guide

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

Fish integrates with Go workspaces and modules (`go.mod`).

---

## Detection & Discovery

Fish automatically detects Go projects when a `go.mod` file is present in the package directory.

### Supported Tasks:
- `build`: Invokes `go build` for binary packages or compiles libraries.
- `check`: Invokes `go vet ./...` for static analysis.
- `test`: Invokes `go test ./...` with caching.

---

## AST Dependency Inference

Fish scans Go source files for `import (...)` statements, automatically resolving module boundaries and linking them into the unified polyglot DAG without manual configuration.

---

## Example `fish.toml`

```toml
[build]
backend = "go"
jobs = 4

[pipelines.build]
inputs = ["**/*.go", "go.mod", "go.sum"]
outputs = ["bin/*"]
```
