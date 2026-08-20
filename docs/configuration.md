# Fish Configuration Guide

> Ã°Å¸Å’Â **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../TRANSLATION.md).

This guide describes how to configure Fish workspaces using `fish.toml`.

---

## Configuration File Overview

Fish reads project configuration from a `fish.toml` file located in the root of your workspace. If no `fish.toml` is present, Fish applies sensible defaults automatically.

```toml
[build]
backend = "rust"
jobs = 8
no_cache = false
sandbox = false
semantic = true
critical_path = true
ram_limit = 85

[cache]
dir = "~/.Fish/cache"
reflink = true

[remote]
cache_url = "http://127.0.0.1:8080"
token = "secret-cache-token"

[daemon]
port = 9527

[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "Cargo.toml", "Cargo.lock"]
outputs = ["target/release/**/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]
```

---

## Top-Level Sections

### `[build]` Ã¢â‚¬â€ Execution Settings

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `backend` | string | Auto | Primary toolchain backend (`rust`, `ts`, `go`, `cc`, `python`, `java`, `dotnet`, `docker`). |
| `jobs` | integer | `num_cpus` | Maximum concurrent worker tasks. |
| `no_cache` | boolean | `false` | Disable local and remote cache lookup. |
| `sandbox` | boolean | `false` | Execute tasks in isolated sandbox environments. |
| `semantic` | boolean | `false` | Enable AST semantic change detection. |
| `critical_path` | boolean | `false` | Prioritize bottlenecks on the dependency graph critical path. |
| `ram_limit` | integer (1-100) | `85` | Throttle concurrency when available system memory drops below this percentage. |
| `timeout` | integer | None | Task execution timeout in seconds. |

---

### `[cache]` Ã¢â‚¬â€ Local Storage Settings

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `dir` | string | `~/.Fish/cache` | Path to the local Content-Addressable Storage (CAS) directory. |
| `reflink` | boolean | `true` | Use Copy-on-Write (CoW) extents or hardlinks to materialize artifacts without I/O copy. |

---

### `[remote]` Ã¢â‚¬â€ Distributed Cache & Execution

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `cache_url` | string | None | Remote cache server endpoint (HTTP/gRPC). |
| `token` | string | None | Authentication bearer token for remote operations. |
| `workers` | list of strings | `[]` | List of remote worker cluster endpoints (e.g. `["worker1:9000", "worker2:9000"]`). |
| `send_source` | boolean | `false` | Compress and transmit source snapshots to workers without shared filesystems. |

---

### `[daemon]` Ã¢â‚¬â€ Background IPC Service

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `port` | integer | `9527` | Loopback TCP port for the Fish background daemon. |

---

### `[pipelines.<task>]` Ã¢â‚¬â€ Task Pipeline Topology

Configure dependencies and caching boundaries between tasks across packages:

```toml
[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "Cargo.toml"]
outputs = ["target/release/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]
```

- **`^build`**: Topological dependency rule. Ensures that all dependency packages run their `build` task before the dependent package starts.
- **`inputs`**: Micro-glob patterns. Only files matching these patterns affect the task fingerprint hash.
- **`outputs`**: File paths to capture into the Content-Addressable Storage (CAS) upon task success.

---

## Environment Variable Overrides

Configuration settings can be overridden via environment variables:

| Variable | Overrides |
| :--- | :--- |
| `fish_CACHE_DIR` | `cache.dir` |
| `fish_JOBS` | `build.jobs` |
| `fish_REMOTE_CACHE` | `remote.cache_url` |
| `fish_REMOTE_TOKEN` | `remote.token` |
| `fish_RAM_LIMIT` | `build.ram_limit` |
| `fish_DAEMON_PORT` | `daemon.port` |
