# Forge CLI Reference

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../TRANSLATION.md).

Complete reference for all Forge command-line interface commands and options.

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

### `forge init`
Initializes Forge configuration and scans the workspace to generate multi-language task definitions (`forge.yaml`).

```bash
forge init [--force]
```

---

### `forge ui`
Starts the real-time interactive Web Dashboard & SVG DAG visualizer with 5-language telemetry (English, Vietnamese, Simplified Chinese, Traditional Chinese, Japanese).

```bash
forge ui [--port <PORT>] [--open]
```

---

### `forge build`
Executes build tasks for packages in the workspace.

```bash
forge build [OPTIONS]
```

**Common Flags:**
- `-p, --package <NAME>`: Build a specific package.
- `--explain`: Diagnose why packages are rebuilt.
- `--profile [FILE]`: Generate a Chrome trace JSON profile.
- `--sandbox`: Run in an isolated sandbox.
- `--ram-limit <PCT>`: Throttle concurrency at memory threshold.

---

### `forge check`
Performs type checking and static analysis without linking full artifacts.

```bash
forge check [OPTIONS]
```

---

### `forge test`
Executes test suites across workspace packages.

```bash
forge test [OPTIONS]
```

---

### `forge run`
Builds and runs a selected binary target.

```bash
forge run -p <PACKAGE> --bin <BINARY> [-- <ARGS>...]
```

---

### `forge query <EXPR>`
Evaluates algebraic queries over the workspace dependency graph.

```bash
forge query "<EXPRESSION>"
```

**Supported Functions:**
- `deps(//pkg)`: All transitive dependencies of `//pkg`.
- `rdeps(//pkg)`: All transitive reverse dependencies of `//pkg`.
- `allpaths(//from, //to)`: All paths between `//from` and `//to`.
- `somepath(//from, //to)`: Shortest path between `//from` and `//to`.
- `filter('pattern', expr)`: Filter matching packages by keyword or pattern.

**Examples:**
```bash
# Find everything required to build forge-cli
forge query "deps(//forge-cli)"

# Find all crates affected by a change in forge-graph
forge query "rdeps(//forge-graph)"

# Find shortest dependency chain between app and util
forge query "somepath(//app, //util)"
```

---

### `forge daemon`
Manages the background build daemon for instant warm graph resolutions.

```bash
# Start the daemon
forge daemon start [--port 9527]

# Check daemon status
forge daemon status [--port 9527]

# Stop the daemon
forge daemon stop [--port 9527]
```

---

### `forge graph`
Prints or exports the project dependency graph.

```bash
forge graph [--format <tree|dot|json>]
```

---

### `forge affected`
Identifies and executes tasks only on packages modified since a Git revision.

```bash
forge affected --since <GIT_REF> [--mode <build|check|test>]
```

---

### `forge cache`
Manages local Content-Addressable Storage (CAS) and fingerprints.

```bash
# Display cache size and object count
forge cache stats

# Remove stale fingerprints and orphaned artifacts
forge cache prune

# Inspect CAS storage
forge cache cas stats
forge cache cas list
```

---

### `forge doctor`
Checks system toolchains, compilers, linkers, and dependencies for readiness.

```bash
forge doctor [--fix] [--ai]
```

---

### `forge ci init` / `forge ci export`
Generates CI workflow configurations for various platforms.

```bash
forge ci init --platform <github|gitlab|circleci|bitbucket|all>
```
