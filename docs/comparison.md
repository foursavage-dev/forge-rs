# Comparison Matrix: Fish vs Other Build Systems

Fish is engineered from the ground up in Rust 2024 for modern polyglot monorepos. Here is a direct, comprehensive comparison with Bazel, Turborepo, and Buck2:

| Capability / Dimension | Fish | Turborepo | Bazel | Buck2 |
| :--- | :--- | :--- | :--- | :--- |
| **Primary Language** | Rust 2024 | Go / Rust | Java / C++ | Rust |
| **Language Support** | Polyglot (11+ native toolchains) | JS / TS focused | Polyglot (starlark rules) | Polyglot (starlark rules) |
| **Configuration Model** | Unified `fish.toml` / auto-detect | `turbo.json` | `WORKSPACE` + `BUILD.bazel` | `BUCK` files |
| **Setup Complexity** | Zero-config / Low | Low | Very High | High |
| **Hashing Engine** | Blake3 (fastest multithreaded) | SHA-256 | SHA-256 | Blake3 / SHA-256 |
| **CAS Compression** | Zstandard (Zstd L1-L22) + CoW | Tar.gz / Gzip | Zstd / Custom | Zstd / Custom |
| **Local Materialization** | Reflink / Copy-on-Write (0ms) | File copy | Symlinks / Hardlinks | Reflink / CoW |
| **Content Chunking** | FastCDC (16KB - 256KB block dedup) | Whole artifact archive | Whole artifact archive | Chunked CAS |
| **VFS Resolution** | In-Memory Snapshot Tree (<2ms) | FS scan | Inotify / Watchman daemon | Watchman / EdenFS |
| **Semantic Invalidation** | AST Interface Hash (.rmeta / ABI) | File hash only | Header-only compile | Header / rmeta compile |
| **AI Diagnostics** | Native IPC + Heuristics + Error Explainer | None | None | None |
| **Interactive Dashboard** | Built-in Web GUI + TUI | Vercel Web App | Third-party UI | Open-source console |

---

## Detailed Architectural Breakdown

### Fish vs Turborepo
* **Language Scope:** Turborepo was created primarily for JavaScript/TypeScript monorepos. While it can run external bash scripts, it has no native understanding of Cargo, Go modules, CMake, or Docker layers. Fish treats all 11+ toolchains as first-class citizens.
* **Storage Performance:** Turborepo uses standard tarballs and file copy. Fish uses Reflink / CoW filesystem extents and FastCDC content-defined chunking to eliminate redundant I/O copy overhead.

### Fish vs Bazel
* **Ergonomics & Ramp-up:** Bazel requires writing complex `BUILD.bazel` files in every subdirectory and managing toolchains manually. Fish provides auto-detection with `fish init` and optional Starlark plugins when customization is desired.
* **Resource Footprint:** Bazel requires a Java runtime daemon with high memory overhead. Fish is a single native static binary with a lightweight footprint.

### Fish vs Buck2
* **Ecosystem Simplicity:** Buck2 is powerful but optimized specifically for Meta's internal workflows and requires external Watchman daemons. Fish bundles its In-Memory VFS, GNU Jobserver, Fast Linkers, and AI engine into a single unified toolkit.
