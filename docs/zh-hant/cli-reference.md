# Fish CLI Reference

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

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

### `fish init`
Initializes Fish configuration and scans the workspace to generate multi-language task definitions (`fish.yaml`).

```bash
fish init [--force]
```

---

### `fish ui`
Starts the real-time interactive Web Dashboard & SVG DAG visualizer with 5-language telemetry (English, Vietnamese, Simplified Chinese, Traditional Chinese, Japanese).

```bash
fish ui [--port <PORT>] [--open]
```

---

### `fish build`
Executes build tasks for packages in the workspace.

```bash
fish build [OPTIONS]
```

**Common Flags:**
- `-p, --package <NAME>`: Build a specific package.
- `--explain`: Diagnose why packages are rebuilt.
- `--profile [FILE]`: Generate a Chrome trace JSON profile.
- `--sandbox`: Run in an isolated sandbox.
- `--ram-limit <PCT>`: Throttle concurrency at memory threshold.

---

### `fish check`
Performs type checking and static analysis without linking full artifacts.

```bash
fish check [OPTIONS]
```

---

### `fish test`
Executes test suites across workspace packages.

```bash
fish test [OPTIONS]
```

---

### `fish run`
Builds and runs a selected binary target.

```bash
fish run -p <PACKAGE> --bin <BINARY> [-- <ARGS>...]
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

### `fish graph`
Prints or exports the project dependency graph.

```bash
fish graph [--format <tree|dot|json>]
```

---

### `fish affected`
Identifies and executes tasks only on packages modified since a Git revision.

```bash
fish affected --since <GIT_REF> [--mode <build|check|test>]
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

### `fish doctor`
Checks system toolchains, compilers, linkers, and dependencies for readiness.

```bash
fish doctor [--fix] [--ai]
```

---

### `fish ci init` / `fish ci export`
Generates CI workflow configurations for various platforms.

```bash
fish ci init --platform <github|gitlab|circleci|bitbucket|all>
```
