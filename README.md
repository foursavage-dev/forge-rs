# Forge 🦀

> A fast, flexible, cache-first build orchestration system, built in Rust.
> Forge works alongside Cargo today and is designed to become a polyglot,
> incremental, cache-aware, eventually-distributed build engine.

Forge is **not** a compiler, a package registry, or a Cargo replacement. It
orchestrates existing toolchains (rustc, cargo, clang, gcc, ...) behind a
build graph, scheduler, executor and cache. For Rust projects it consumes
official `cargo metadata` output instead of re-parsing `Cargo.toml`.

## Status

Milestones 1–4 are in place: Forge discovers a Cargo project, loads its
metadata, builds the workspace graph (and a test graph that includes
dev-dependency edges), maps it onto fingerprinted tasks, and executes them
in parallel with a fingerprint cache that makes rebuilds instant when
nothing changed. The same pipeline drives C/C++ (`forge.cc.json`), Go
(`forge.go.json`/`go.mod`) and TypeScript/JavaScript (`forge.ts.json`/
`package.json`) projects. `forge build`, `forge check`, `forge test`,
`forge run`, `forge graph`, `forge clean` and `forge watch` all run real
toolchains, and this repository builds itself with `forge build` on CI.
Builds can be sandboxed to a clean environment, tasks can be killed after
a timeout, and every run can dump a Chrome trace for profiling.

```text
$ forge build
Cache:                C:\Users\you\.forge\cache
Forge 🦀 0.1.0

Project:              (workspace)
Manifest:             C:\work\forge-rs\Cargo.toml
Workspace:            forge-rs (workspace)
Workspace packages:   12 (12 default)

Build graph:
             forge-graph          forge-executor
                  ↓                 ↓
              forge-core             forge-cache         forge-scheduler
                   ↓                  ↓                ↓
        forge-backend-cc        forge-backend-go      forge-backend-rust        forge-backend-ts           forge-sandbox      forge-remote-cache
                ↓                ↓               ↓                ↓                 ↓               ↓
               forge-cli

Building...

✓ forge-executor, forge-graph
✓ forge-cache, forge-core, forge-scheduler
✓ forge-backend-cc, forge-backend-go, forge-backend-rust, forge-backend-ts, forge-remote-cache, forge-sandbox
✓ forge-cli
Build completed successfully.
  Tasks:     4 total
  Executed:  4
  Cached:    0
  Failed:    0
  Workers:   8
  Duration:  78.47s
  Cache:     0 hits, 4 misses, 0 errors
```

Run the same command again and the tasks are skipped from cache; change
`forge-core` and both `forge-core` and `forge-cli` rebuild, because a
package's fingerprint folds in the fingerprints of everything it depends on
(a change anywhere in a dependency cone invalidates the whole cone).

The workspace's 12 packages are not built one cargo invocation each:
packages with no dependencies on each other are batched into a single
`cargo build --package a --package b ...` task per build level (4 levels
here, so 4 tasks). Cargo still schedules within each level in parallel and
builds get far fewer process spawns, which is what makes the cold build
above complete in 78s instead of a dozen serialized `cargo` startups.

## Layout

```text
forge/
├── Cargo.toml                 # workspace root
├── forge.toml                 # forge's own project configuration
├── .github/workflows/         # dogfood CI: forge builds forge
├── crates/
│   ├── forge-core/            # project discovery, Cargo metadata model
│   ├── forge-graph/           # build graph: nodes, edges, states, topo order
│   ├── forge-executor/        # task model, CommandSpec, process execution
│   ├── forge-scheduler/       # parallel ready-queue scheduler
│   ├── forge-cache/           # fingerprint store + caching executor wrapper
│   ├── forge-backend-rust/    # Cargo metadata → task graph + fingerprints
│   ├── forge-backend-cc/      # C/C++ backend (gcc/clang/msvc, forge.cc.json)
│   ├── forge-backend-go/      # Go backend (go.mod, forge.go.json)
│   ├── forge-backend-ts/      # TypeScript/JS backend (forge.ts.json, package.json)
│   ├── forge-sandbox/         # hermetic environment sanitization
│   ├── forge-remote-cache/    # remote cache client + tiered L1/L2 composite
│   └── forge-cli/             # the `forge` binary
└── tests/                     # integration tests live next to each crate
```

The build graph (`forge-graph`) is generic over its node payload, keeps a
`TaskState` per node (pending/ready/running/succeeded/failed/skipped/cached/
cancelled), rejects dependency cycles, and offers topological order, level
partitioning and failure propagation (a failed task cancels its transitive
dependents). Cargo dev-dependency edges are excluded from the build graph:
Cargo allows cycles through them, and they only matter for test builds.

The scheduler (`forge-scheduler`) runs ready tasks on a fixed worker pool,
never spawns processes itself (it only talks to the `TaskExecutor` trait),
counts cached tasks, cancels the dependents of failed tasks, and reports a
`BuildSummary` (`-j` sets the worker count).

The Rust backend (`forge-backend-rust`) partitions the workspace package
graph into build levels and maps every level to one `cargo build --package
X --package Y ...` task (so a workspace with 12 packages and 4 levels runs
4 cargo processes, not 12), and fingerprints each package's own inputs —
file contents (mtime-blind), the workspace `Cargo.lock`, the toolchain
versions, and the fingerprint of every direct dependency, combined in a
content hash. `forge check` is the same pipeline with `cargo check`, and
`forge test` runs `cargo test --package X ...` over a graph that includes
dev-dependency edges, so dev-only packages build before the tests that use
them.

The cache (`forge-cache`) stores `(key, fingerprint)` records under
`~/.forge/cache` with atomic writes; a missing or corrupt record is simply a
miss, and cache failures never fail a build. `forge clean` delegates to
`cargo clean`.

## Building and testing

```bash
cargo build --workspace      # builds the `forge` binary
cargo test --workspace       # unit + integration tests (requires cargo on PATH)
cargo clippy --workspace --all-targets
```

The binary is `target/debug/forge`.

## Commands

| Command                            | Status                              |
| ---------------------------------- | ----------------------------------- |
| `forge --version` / `forge version`| implemented                         |
| `forge build [-j N] [-v] [--no-cache] [--sandbox] [--timeout SECS] [--profile FILE] [--remote-cache ADDR] [--remote-workers ADDRS] [PATH]` | implemented |
| `forge check [-j N] [-v] [--no-cache] [--sandbox] [--timeout SECS] [--profile FILE] [PATH]` | implemented |
| `forge test [-j N] [-v] [--no-cache] [--sandbox] [--timeout SECS] [--profile FILE] [PATH]`  | implemented |
| `forge run [-p PKG] [--bin BIN] [ARGS]`       | implemented                |
| `forge graph [--format tree\|json\|dot] [PATH]`| implemented                |
| `forge watch [--mode build\|check\|test] [--debounce MS] [--clear] [--once]` | implemented |
| `forge affected [--since REV] [--mode build\|check\|test]` | implemented |
| `forge doctor`                        | implemented                        |
| `forge cache stats`                   | implemented                        |
| `forge cache prune [--older-than DUR] [--max-size SIZE] [--dir PATH]` | implemented |
| `forge cache-server [--listen ADDR] [--dir PATH] [--auth-token TOKEN]` | implemented |
| `forge worker [--listen ADDR] [--auth-token TOKEN] [--name NAME] [--max-concurrency N]` | implemented |
| `forge clean [PATH]`               | implemented (cargo clean)           |

`forge build` is a single entry point for four backends: a Cargo workspace
(`Cargo.toml`), a C/C++ project (`forge.cc.json`), a Go module
(`forge.go.json`, or plain `go.mod`), or a TypeScript/JavaScript project
(`forge.ts.json`, or plain `package.json` whose npm scripts become
`<name>:<script>` tasks). Per-project defaults live in an optional
`forge.toml` (`backend`, `jobs`, `no_cache`, `sandbox`, `timeout`,
`profile`); flags always win.

`forge test` builds a *test graph* that also honors dev-dependency edges
(cyclic dev-dependency edges are dropped rather than rejected), then runs
`cargo test --package X ...` per build level and reports failed assertions
from the captured output. `forge run` builds first, then delegates to
`cargo run`. `forge graph` renders the dependency graph as a tree, JSON, or
Graphviz dot.

`forge watch` re-runs the chosen mode whenever a project file changes
(debounced, with `--clear` to wipe the terminal between runs). `--sandbox`
strips the inherited environment from every spawned tool, `--timeout SECS`
kills a task — and its whole process tree — when it overruns, and
`--profile FILE` writes the run as a Chrome trace with a `critical_path`
event for bottleneck analysis.

## Forge builds Forge

This repository is its own first customer. The root `forge.toml` pins its
preferences, and the dogfood CI (`.github/workflows/dogfood.yml`) builds the
tree with the `forge` binary it just compiled:

```bash
cargo build --release            # bootstrap forge
./target/release/forge build     # then forge builds the whole workspace
./target/release/forge test      # build + run every package's tests
./target/release/forge graph --format dot
```

Real output of `forge build` on this repository (12 packages, 4 build
levels, 8 workers, Windows host — the running binary is the release one,
so it can safely overwrite the debug one it is building):

```text
Cache:                C:\Users\you\.forge\cache
Forge 🦀 0.1.0

Project:              (workspace)
Manifest:             C:\work\forge-rs\Cargo.toml
Workspace:            forge-rs (workspace)
Workspace packages:   12 (12 default)

Build graph:
             forge-graph          forge-executor
                  ↓                 ↓
              forge-core             forge-cache         forge-scheduler
                   ↓                  ↓                ↓
        forge-backend-cc        forge-backend-go      forge-backend-rust        forge-backend-ts           forge-sandbox      forge-remote-cache
                ↓                ↓               ↓                ↓                 ↓               ↓
               forge-cli

Building...

✓ forge-executor, forge-graph
✓ forge-cache, forge-core, forge-scheduler
✓ forge-backend-cc, forge-backend-go, forge-backend-rust, forge-backend-ts, forge-remote-cache, forge-sandbox
✓ forge-cli
Build completed successfully.
  Tasks:     4 total
  Executed:  4
  Cached:    0
  Failed:    0
  Workers:   8
  Duration:  78.47s
  Cache:     0 hits, 4 misses, 0 errors
```

Run the same command again and the fingerprint cache skips every unchanged
package — all four levels come from cache in 0.00s. Touch `forge-core` and
`forge-core`, its dependents, and everything above it rebuild.

## Roadmap

1. **Milestone 1:** workspace, CLI, core — Cargo project detection and
   metadata. ✔
2. **Milestone 2:** build graph — nodes, edges, task states, topological
   order, failure propagation. ✔
3. **Milestone 3:** scheduler + executor + incremental cache — parallel
   builds of real Rust projects with content fingerprints. ✔
4. **Milestone 4:** polyglot — C/C++, Go and TypeScript backends, `forge
   run`, `forge graph`, `forge watch`, per-project `forge.toml`, dogfood
   CI. ✔
5. **Milestone 5:** remote cache daemon & tiered composite caching (`forge cache-server`, `--remote-cache`), distributed execution cluster & failover (`forge worker`, `--remote-workers`). ✔
6. Then: distributed artifact CAS storage, dynamic worker discovery, and remote containerized sandboxing.

## Design principles

- Rust-first, performance-oriented, low memory usage, cross-platform.
- Modular: every major subsystem is its own crate behind a trait boundary.
- No language-specific logic in core — backends implement a trait interface.
- Deterministic where possible, honest about what is not implemented.
- Small, high-quality dependency tree; no unnecessary reimplementations.

## License

MIT — see [LICENSE](LICENSE).