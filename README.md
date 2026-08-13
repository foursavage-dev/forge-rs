# Forge 🦀

> A fast, flexible, cache-first build orchestration system, built in Rust.
> Forge works alongside Cargo today and is designed to become a polyglot,
> incremental, cache-aware, eventually-distributed build engine.

Forge is **not** a compiler, a package registry, or a Cargo replacement. It
orchestrates existing toolchains (rustc, cargo, clang, gcc, ...) behind a
build graph, scheduler, executor and cache. For Rust projects it consumes
official `cargo metadata` output instead of re-parsing `Cargo.toml`.

## Status

Milestones 1–3 are in place: Forge discovers a Cargo project, loads its
metadata, builds the workspace graph (and a test graph that includes
dev-dependency edges), maps it onto fingerprinted tasks, and executes them
in parallel with a fingerprint cache that makes rebuilds instant when
nothing changed. `forge build`, `forge check`, `forge test` and
`forge clean` all run real Cargo commands.

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
├── crates/
│   ├── forge-core/            # project discovery, Cargo metadata model
│   ├── forge-graph/           # build graph: nodes, edges, states, topo order
│   ├── forge-executor/        # task model, CommandSpec, process execution
│   ├── forge-scheduler/       # parallel ready-queue scheduler
│   ├── forge-cache/           # fingerprint store + caching executor wrapper
│   ├── forge-backend-rust/    # Cargo metadata → task graph + fingerprints
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
| `forge clean [PATH]`               | implemented (cargo clean)           |
| `forge run/graph/cache/doctor/...` | planned                             |

`forge test` builds a *test graph* that also honors dev-dependency edges
(cyclic dev-dependency edges are dropped rather than rejected), then runs
`cargo test --package X` per workspace package and reports failed assertions
from the captured output.

## Roadmap

1. **Milestone 1:** workspace, CLI, core — Cargo project detection and
   metadata. ✔
2. **Milestone 2:** build graph — nodes, edges, task states, topological
   order, failure propagation. ✔
3. **Milestone 3:** scheduler + executor + incremental cache — parallel
   builds of real Rust projects with content fingerprints. ✔
4. Then: task-level concurrency analytics and critical-path reporting,
   C/C++ and Go backends, backend plugin API, artifact caching, remote
   cache, sandboxing, remote execution, distributed builds.

## Design principles

- Rust-first, performance-oriented, low memory usage, cross-platform.
- Modular: every major subsystem is its own crate behind a trait boundary.
- No language-specific logic in core — backends implement a trait interface.
- Deterministic where possible, honest about what is not implemented.
- Small, high-quality dependency tree; no unnecessary reimplementations.

## License

MIT — see [LICENSE](LICENSE).