# Fish CLI Reference

> Ã°Å¸Å’Â **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../TRANSLATION.md).

Complete reference for all Fish command-line interface commands and options.

---

## Global Options

- `--experimental`: Enable experimental features.
- `-v, --verbose`: Enable verbose diagnostic output.
- `-j, --jobs <N>`: Maximum parallel worker threads.
- `--no-cache`: Bypass both local and remote caches.
- `--cache-dir <PATH>`: Custom local cache directory.
- `--explain`: Print detailed rebuild reasons for dirty targets.
- `--pgo-generate`: Instrument binaries for Profile-Guided Optimization.
- `--pgo-use`: Compile binaries utilizing gathered PGO profile data.

---

## Primary Commands

### `Fish init`
Initializes Fish configuration and scans the workspace to generate multi-language task definitions (`fish.yaml`).

```bash
Fish init [--force]
```

---

### `Fish ui`
Starts the real-time interactive Web Dashboard & SVG DAG visualizer with 5-language telemetry (English, Vietnamese, Simplified Chinese, Traditional Chinese, Japanese).

```bash
Fish ui [--port <PORT>] [--open]
```

---

### `Fish build`
Executes build tasks for packages in the workspace.

```bash
Fish build [OPTIONS]
```

**Common Flags:**
- `-p, --package <NAME>`: Build a specific package.
- `--explain`: Diagnose why packages are rebuilt.
- `--profile [FILE]`: Generate a Chrome trace JSON profile.
- `--sandbox`: Run in an isolated sandbox.
- `--ram-limit <PCT>`: Throttle concurrency at memory threshold.

---

### `Fish check`
Performs type checking and static analysis without linking full artifacts.

```bash
Fish check [OPTIONS]
```

---

### `Fish test`
Executes test suites across workspace packages.

```bash
Fish test [OPTIONS]
```

---

### `Fish run`
Builds and runs a selected binary target.

```bash
Fish run -p <PACKAGE> --bin <BINARY> [-- <ARGS>...]
```

---

### `Fish query <EXPR>`
Evaluates algebraic queries over the workspace dependency graph.

```bash
Fish query "<EXPRESSION>"
```

**Supported Functions:**
- `deps(//pkg)`: All transitive dependencies of `//pkg`.
- `rdeps(//pkg)`: All transitive reverse dependencies of `//pkg`.
- `allpaths(//from, //to)`: All paths between `//from` and `//to`.
- `somepath(//from, //to)`: Shortest path between `//from` and `//to`.
- `filter('pattern', expr)`: Filter matching packages by keyword or pattern.

**Examples:**
```bash
# Find everything required to build fish-cli
Fish query "deps(//fish-cli)"

# Find all crates affected by a change in fish-graph
Fish query "rdeps(//fish-graph)"

# Find shortest dependency chain between app and util
Fish query "somepath(//app, //util)"
```

---

### `Fish daemon`
Manages the background build daemon for instant warm graph resolutions.

```bash
# Start the daemon
Fish daemon start [--port 9527]

# Check daemon status
Fish daemon status [--port 9527]

# Stop the daemon
Fish daemon stop [--port 9527]
```

---

### `Fish graph`
Prints or exports the project dependency graph.

```bash
Fish graph [--format <tree|dot|json>]
```

---

### `Fish affected`
Identifies and executes tasks only on packages modified since a Git revision.

```bash
Fish affected --since <GIT_REF> [--mode <build|check|test>]
```

---

### `Fish cache`
Manages local Content-Addressable Storage (CAS) and fingerprints.

```bash
# Display cache size and object count
Fish cache stats

# Remove stale fingerprints and orphaned artifacts
Fish cache prune

# Inspect CAS storage
Fish cache cas stats
Fish cache cas list
```

---

### `Fish doctor`
Checks system toolchains, compilers, linkers, and dependencies for readiness.

```bash
Fish doctor [--fix] [--ai]
```

---

### `Fish ci init` / `Fish ci export`
Generates CI workflow configurations for various platforms.

```bash
Fish ci init --platform <github|gitlab|circleci|bitbucket|all>
```
