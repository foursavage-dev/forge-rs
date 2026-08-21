# Migration Guides to Fish

This guide explains how to migrate existing build configurations from Turborepo, Nx, and Bazel into a unified `fish.toml`.

---

## 1. Migrating from Turborepo (`turbo.json`)

Turborepo configurations define pipelines with dependencies and outputs. In Fish, these translate directly into `[pipelines]`.

### Before: `turbo.json`
```json
{
  "$schema": "https://turbo.build/schema.json",
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": [".next/**", "dist/**"]
    },
    "test": {
      "dependsOn": ["build"],
      "outputs": []
    }
  }
}
```

### After: `fish.toml`
```toml
[build]
backend = "ts"
jobs = 8
reflink = true
semantic = true

[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "package.json"]
outputs = [".next/**", "dist/**"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*"]
outputs = []
```

---

## 2. Migrating from Nx (`nx.json`)

Nx targets and caching rules map directly to Fish task definitions:

### Before: `nx.json`
```json
{
  "targetDefaults": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": ["{projectRoot}/dist"]
    }
  }
}
```

### After: `fish.toml`
```toml
[build]
backend = "ts"

[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*"]
outputs = ["dist/**/*"]
```

---

## 3. Migrating from Bazel (`WORKSPACE` / `BUILD.bazel`)

In Bazel, dependencies are declared using explicit labels like `//packages/core:core`. In Fish, dependencies across languages are auto-discovered from package manifests (`Cargo.toml`, `go.mod`, `package.json`, `CMakeLists.txt`).

Run the automated migration assistant:
```bash
fish init --force
fish doctor --fix
```
