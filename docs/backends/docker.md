# Docker Backend Guide

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../../TRANSLATION.md).

Forge treats Docker images as first-class build artifacts within your workspace DAG.

---

## Detection & Discovery

Forge detects Docker components when a `Dockerfile` or `Containerfile` is present in a package directory.

---

## Dependency Chaining

Docker build tasks can declare dependencies on upstream compilation outputs:

```toml
[pipelines.docker]
depends_on = ["^build"]
inputs = ["Dockerfile", "target/release/app"]
```

Forge guarantees that all binaries and compiled assets are produced, validated, and placed in the build context before invoking `docker build` or BuildKit.
