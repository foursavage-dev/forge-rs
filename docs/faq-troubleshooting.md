# Frequently Asked Questions & Troubleshooting

> Ã°Å¸Å’Â **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../TRANSLATION.md).

This document covers common questions, migration recipes, and troubleshooting steps for Fish.

---

## Frequently Asked Questions (FAQ)

### 1. Does Fish replace Cargo, npm, or go build?
No. Fish is a build **orchestrator**, not a compiler replacement. It coordinates your existing toolchains (Cargo, rustc, Node.js, Go, GCC/Clang, dotnet), analyzes the unified dependency graph, and accelerates builds using hermetic caching, parallel scheduling, and remote execution.

### 2. How do I migrate an existing monorepo to Fish?
Fish automatically discovers projects from their manifests (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, `CMakeLists.txt`, `pom.xml`, `*.csproj`).
1. Navigate to your project root.
2. Run `Fish build` to let Fish discover your workspace.
3. (Optional) Create a `fish.toml` in your root directory to customize pipeline dependencies and cache paths.

### 3. How does Fish's CAS caching work?
Fish computes Blake3 fingerprints over input files, toolchain versions, and environment variables. When a task produces output artifacts, they are compressed with Zstandard and stored in a Content-Addressable Storage (CAS) directory (`~/.Fish/cache`). If inputs do not change, Fish materializes artifacts instantly using copy-on-write extents or hardlinks without re-executing compilers.

---

## Troubleshooting Recipes

### Issue: Target is rebuilding unexpectedly
**Solution:**
Use the `--explain` flag to see why a target was considered dirty:
```bash
Fish build --explain
```
Common causes include:
- A source file was recently touched.
- An upstream dependency's output hash changed.
- An environment variable difference invalidated the cache.

---

### Issue: High RAM usage during parallel builds
**Solution:**
When building multiple large crates or C++ modules concurrently, memory pressure can cause disk swapping. Use the `--ram-limit` flag or configure `ram_limit` in `fish.toml`:
```bash
Fish build --ram-limit 80
```
Fish's resource governor will automatically throttle concurrency whenever memory usage crosses the threshold.

---

### Issue: Background daemon port conflict (`9527`)
**Solution:**
If port `9527` is in use by another process, specify a custom port:
```bash
Fish daemon start --port 9588
```
Or set the environment variable:
```bash
export fish_DAEMON_PORT=9588
```

---

### Issue: File lock error on Windows (`os error 5: Access is denied`)
**Solution:**
On Windows, running a binary from within the `target/debug` directory locks the executable file on disk. Install Fish globally to `%USERPROFILE%\.cargo\bin`:
```bash
cargo install --path crates/fish-cli --force
```
Then invoke `Fish` directly from any directory.
