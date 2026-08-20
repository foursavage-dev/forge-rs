---
layout: home

hero:
  name: "Fish"
  text: "High-Performance Polyglot Build Orchestration"
  tagline: "Unify builds across 11+ toolchains with algebraic DAG scheduling, deterministic CAS caching, and distributed racing."
  image:
    src: /logo.svg
    alt: Fish Logo
  actions:
    - theme: brand
      text: Get Started
      link: /getting-started
    - theme: alt
      text: Architecture
      link: /architecture
    - theme: alt
      text: View on GitHub
      link: https://github.com/requla11/fish

features:
  - icon: ⚡
    title: Extreme Concurrency & Racing
    details: GNU Jobserver integration, dynamic remote racing, and bin-packing worker queues maximize CPU and network utilization.
  - icon: 🎯
    title: 11+ Language Toolchains
    details: Native zero-config auto-discovery for Rust, Go, TypeScript, Python, C/C++, Docker, Java, .NET, Swift, Dart, and Zig.
  - icon: 🔒
    title: Deterministic Cache & CAS
    details: Blake3 multi-tier fingerprinting with ZSTD compression and two-phase pruning delivers sub-millisecond cache hits.
  - icon: 🌐
    title: Interactive Web Dashboard
    details: Real-time DAG visualization, live build telemetry, and multi-language metrics monitor all monorepo workloads.
  - icon: 🛡️
    title: Cryptographic SBOM & Sandbox
    details: Automated dependency vulnerability scanning, hermetic sandboxing, and Ed25519 artifact signing built-in.
  - icon: 🚀
    title: Algebraic DAG Query Engine
    details: Query dependencies, reverse dependencies, and critical execution paths with expressive algebraic expressions.
---
