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
nothing changed. The same pipeline drives C/C++ (`forge.cc.json`) and Go
(`forge.go.json`/`go.mod`) projects. `forge build`, `forge check`,
`forge test`, `forge run`, `forge graph` and `forge clean` all run real
toolchains, and this repository builds itself with `forge build` on CI.

```text
$ forge build
Cache:                C:\Users\you\.forge\cache
Forge 🦀 0.1.0

Project:              (workspace)
Manifest:             C:\work\forge-rs\Cargo.toml
Workspace:            forge-rs (workspace)
Workspace packages:   3 (3 default)

Build graph:
      forge-graph
           ↓
       forge-core
            ↓
        forge-cli

Building...

✓ forge-graph
✓ forge-core
✓ forge-cli
Build completed successfully.
  Tasks:     2 total
  Executed:  2
  Cached:    0
  Failed:    0
  Workers:   8
  Duration:  0.93s
  Cache:     0 hits, 2 misses, 0 errors
```

Run the same command again and the tasks are skipped from cache; change
`forge-core` and both `forge-core` and `forge-cli` rebuild, because a
package's fingerprint folds in the fingerprints of everything it depends on
(a change anywhere in a dependency cone invalidates the whole cone).

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

The Rust backend (`forge-backend-rust`) maps every workspace package to a
`cargo build --package X` task and fingerprints each package's own inputs —
file contents (mtime-blind), the workspace `Cargo.lock`, the toolchain
versions, and the fingerprint of every direct dependency, combined in a
content hash. `forge check` is the same pipeline with `cargo check`, and
`forge test` runs `cargo test --package X` over a graph that includes
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
| `forge build [-j N] [-v] [--no-cache] [PATH]` | implemented                |
| `forge check [-j N] [-v] [--no-cache] [PATH]` | implemented              |
| `forge test [-j N] [-v] [--no-cache] [PATH]`  | implemented                |
| `forge run [-p PKG] [--bin BIN] [ARGS]`       | implemented                |
| `forge graph [--format tree\|json\|dot] [PATH]`| implemented                |
| `forge clean [PATH]`               | implemented (cargo clean)           |
| `forge cache/doctor/...`           | planned                             |

`forge build` is a single entry point for three backends: a Cargo workspace
(`Cargo.toml`), a C/C++ project (`forge.cc.json`), or a Go module
(`forge.go.json`, or plain `go.mod`). Per-project defaults live in an
optional `forge.toml` (`backend`, `jobs`, `no_cache`); flags always win.

`forge test` builds a *test graph* that also honors dev-dependency edges
(cyclic dev-dependency edges are dropped rather than rejected), then runs
`cargo test --package X` per workspace package and reports failed assertions
from the captured output. `forge run` builds first, then delegates to
`cargo run`. `forge graph` renders the dependency graph as a tree, JSON, or
Graphviz dot.

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

Real output of `forge build` on this repository (10 packages, 8 workers,
Windows host — the running binary is the release one, so it can safely
overwrite the debug one it is building):

```text
Cache:                C:\Users\you\.forge\cache
Forge 🦀 0.1.0

Project:              (workspace)
Manifest:             C:\work\forge-rs\Cargo.toml
Workspace:            forge-rs (workspace)
Workspace packages:   10 (10 default)

Build graph:
             forge-graph          forge-executor
              forge-core             forge-cache         forge-scheduler
      forge-remote-cache        forge-backend-cc        forge-backend-go      forge-backend-rust
               forge-cli

Building...

✓ forge-graph
✓ forge-executor
✓ forge-core
✓ forge-scheduler
✓ forge-cache
✓ forge-backend-rust
✓ forge-backend-cc
✓ forge-backend-go
✓ forge-remote-cache
✓ forge-cli
Build completed successfully.
  Tasks:     10 total
  Executed:  10
  Cached:    0
  Failed:    0
  Workers:   8
  Duration:  9.74s
```

Run the same command again and the fingerprint cache skips every unchanged
package; touch `forge-core` and both `forge-core` and everything that
depends on it rebuild.

## Roadmap

1. **Milestone 1:** workspace, CLI, core — Cargo project detection and
   metadata. ✔
2. **Milestone 2:** build graph — nodes, edges, task states, topological
   order, failure propagation. ✔
3. **Milestone 3:** scheduler + executor + incremental cache — parallel
   builds of real Rust projects with content fingerprints. ✔
4. **Milestone 4:** polyglot — C/C++ and Go backends, `forge run`, `forge
   graph`, per-project `forge.toml`, dogfood CI. ✔
5. Then: task-level concurrency analytics and critical-path reporting,
   backend plugin API, artifact caching, remote cache server (the client
   scaffold already exists), sandboxing, remote execution, distributed
   builds.

## Design principles

- Rust-first, performance-oriented, low memory usage, cross-platform.
- Modular: every major subsystem is its own crate behind a trait boundary.
- No language-specific logic in core — backends implement a trait interface.
- Deterministic where possible, honest about what is not implemented.
- Small, high-quality dependency tree; no unnecessary reimplementations.

## License

MIT — see [LICENSE](LICENSE).