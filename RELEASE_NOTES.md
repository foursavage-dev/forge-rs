# 🚀 Forge v0.1.0 Release Notes

**Release Tag:** `v0.1.0`  
**Release Date:** August 16, 2026  
**Author:** ForgeBot 🤖  

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

We are thrilled to announce the official release of **Forge v0.1.0**! Forge is a fast, flexible, cache-first build orchestration system designed for high-performance builds across multiple programming languages.

---

## 🚀 What's New / Highlights

- **Windows Setup Installer (`forge-setup-windows-x86_64.exe`):**  
  A native 1-click installer app with full multi-language localization (English, Tiếng Việt, 简体中文, 繁體中文, 日本語), automatic Windows User PATH configuration via registry broadcast, and toolchain environment diagnostics.
- **Polyglot Multi-Language Support:**  
  Robust task DAG resolution, caching, and execution across 11 backends:
  - **Rust** (Cargo workspace & level batching)
  - **C/C++** (Clang, GCC, MSVC)
  - **Go** (go build, test, vet)
  - **TypeScript/JavaScript** (pnpm, yarn, npm, bun, vite, next)
  - **Python** (pyproject.toml, uv, poetry, pipfile, pytest)
  - **Java/Kotlin** (Maven & Gradle)
  - **.NET** (C#, F#, VB, dotnet restore/build/test/publish)
  - **Swift** (SwiftPM & Apple targets)
  - **Dart/Flutter** (pub get, test, analyze, build)
  - **Zig** (build, fetch, test)
  - **Docker** (multi-stage build graph generation)

---

## ⚡ Performance & Core Engine Improvements

- **High-Throughput Stream Hashing:** Integrated fixed 64KB chunk buffer hashing with Blake3 (`FingerprintUtils::hash_file_into`), preventing out-of-memory overhead on large projects.
- **Topological Task Graph Scheduler:** Generic DAG cycle detection and dependency ordering via `TaskDagBuilder`.
- **Standardized Multi-Tier Caching:** Deterministic cache keys across all compiler pipelines (`forge-cache`, `forge-cas`).

---

## 📦 Installation & Download Instructions

### Windows (Setup Installer - Recommended)

Download **[`forge-setup-windows-x86_64.exe`](https://github.com/foursavage-dev/forge-rs/releases/download/v0.1.0/forge-setup-windows-x86_64.exe)** and double-click to install Forge. The installer will automatically add Forge to your User `PATH` environment variable.

### Direct Binary Downloads

| Platform | Download Link |
| :--- | :--- |
| **Windows (Setup Installer)** | [`forge-setup-windows-x86_64.exe`](https://github.com/foursavage-dev/forge-rs/releases/download/v0.1.0/forge-setup-windows-x86_64.exe) |
| **Windows (Portable x86_64)** | [`forge-windows-x86_64.exe`](https://github.com/foursavage-dev/forge-rs/releases/download/v0.1.0/forge-windows-x86_64.exe) |
| **Linux (x86_64)** | [`forge-linux-x86_64`](https://github.com/foursavage-dev/forge-rs/releases/download/v0.1.0/forge-linux-x86_64) |
| **macOS (Apple Silicon M1/M2/M3/M4)** | [`forge-macos-aarch64`](https://github.com/foursavage-dev/forge-rs/releases/download/v0.1.0/forge-macos-aarch64) |
| **macOS (Intel x86_64)** | [`forge-macos-x86_64`](https://github.com/foursavage-dev/forge-rs/releases/download/v0.1.0/forge-macos-x86_64) |
| **Verification Checksums** | [`SHA256SUMS.txt`](https://github.com/foursavage-dev/forge-rs/releases/download/v0.1.0/SHA256SUMS.txt) |

---

## 🛠️ Quick Start

```bash
# Verify installation
forge --help

# Initialize a project
forge init

# Build with smart caching
forge build
```
