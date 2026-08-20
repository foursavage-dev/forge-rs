# 🚀 Fish v0.1.0 Release Notes

**Release Tag:** `v0.1.0`  
**Release Date:** August 16, 2026  
**Author:** FishBot 🤖  

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

We are thrilled to announce the official release of **Fish v0.1.0**! Fish is a fast, flexible, cache-first build orchestration system designed for high-performance builds across multiple programming languages.

---

## 🚀 What's New / Highlights

- **Windows Setup Installer (`fish-setup-windows-x86_64.exe`):**  
  A native 1-click installer app with full multi-language localization (English, Tiáº¿ng Viá»‡t, ç®€ä½“ä¸­æ–‡, ç¹é«”ä¸­æ–‡, æ—¥æœ¬èªž), automatic Windows User PATH configuration via registry broadcast, and toolchain environment diagnostics.
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

## âš¡ Performance & Core Engine Improvements

- **High-Throughput Stream Hashing:** Integrated fixed 64KB chunk buffer hashing with Blake3 (`FingerprintUtils::hash_file_into`), preventing out-of-memory overhead on large projects.
- **Topological Task Graph Scheduler:** Generic DAG cycle detection and dependency ordering via `TaskDagBuilder`.
- **Standardized Multi-Tier Caching:** Deterministic cache keys across all compiler pipelines (`fish-cache`, `fish-cas`).

---

## 📦 Installation & Download Instructions

### Windows (Setup Installer - Recommended)

Download **[`fish-setup-windows-x86_64.exe`](https://github.com/requla11/fish/releases/download/v0.1.0/fish-setup-windows-x86_64.exe)** and double-click to install Fish. The installer will automatically add Fish to your User `PATH` environment variable.

### Direct Binary Downloads

| Platform | Download Link |
| :--- | :--- |
| **Windows (Setup Installer)** | [`fish-setup-windows-x86_64.exe`](https://github.com/requla11/fish/releases/download/v0.1.0/fish-setup-windows-x86_64.exe) |
| **Windows (Portable x86_64)** | [`fish-windows-x86_64.exe`](https://github.com/requla11/fish/releases/download/v0.1.0/fish-windows-x86_64.exe) |
| **Linux (x86_64)** | [`fish-linux-x86_64`](https://github.com/requla11/fish/releases/download/v0.1.0/fish-linux-x86_64) |
| **macOS (Apple Silicon M1/M2/M3/M4)** | [`fish-macos-aarch64`](https://github.com/requla11/fish/releases/download/v0.1.0/fish-macos-aarch64) |
| **macOS (Intel x86_64)** | [`fish-macos-x86_64`](https://github.com/requla11/fish/releases/download/v0.1.0/fish-macos-x86_64) |
| **Verification Checksums** | [`SHA256SUMS.txt`](https://github.com/requla11/fish/releases/download/v0.1.0/SHA256SUMS.txt) |

---

## 🛠️ Quick Start

```bash
# Verify installation
Fish --help

# Initialize a project
Fish init

# Build with smart caching
Fish build
```
