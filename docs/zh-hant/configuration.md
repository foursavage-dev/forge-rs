# Fish 配置參考指南

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](TRANSLATION.md)。

本指南介绍如何使用 `fish.toml` 配置 Fish 工作區。

---

## 配置文件概覽

Fish 会读取工作區根目录下的 `fish.toml` 配置文件。如果未提供 `fish.toml`，Fish 将自动应用一套合理的默认值。

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
dir = "~/.fish/cache"
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

## 顶级配置段

### `[build]` —— 执行参数配置

| 配置键 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `backend` | 字符串 | Auto | 主工具链后端 (`rust`, `ts`, `go`, `cc`, `python`, `java`, `dotnet`, `docker`)。 |
| `jobs` | 整数 | `num_cpus` | 最大并发执行的工作任务数。 |
| `no_cache` | 布尔值 | `false` | 禁用本地和远程缓存查询。 |
| `sandbox` | 布尔值 | `false` | 在沙盒隔离环境中执行构建任务。 |
| `semantic` | 布尔值 | `false` | 启用基于 AST 的语义级变动检测。 |
| `critical_path` | 布尔值 | `false` | 优先执行依赖图关键路径上的瓶颈任务。 |
| `ram_limit` | 整数 (1-100) | `85` | 当系统可用内存低于此百分比时动态节流限制并发数。 |
| `timeout` | 整数 | None | 单个任务的超时时间（秒）。 |

---

### `[cache]` —— 本地存储配置

| 配置键 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `dir` | 字符串 | `~/.fish/cache` | 本地内容寻址存储（CAS）的存储目录路径。 |
| `reflink` | 布尔值 | `true` | 使用写时复制 (CoW) 或硬链接极速实体化产物，免去磁盘 I/O 复制。 |

---

### `[remote]` —— 分布式缓存与执行

| 配置键 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `cache_url` | 字符串 | None | 远程缓存服务器地址 (HTTP/gRPC)。 |
| `token` | 字符串 | None | 远程访问所需的 Bearer 认证 Token。 |
| `workers` | 字符串列表 | `[]` | 远程执行集群 Worker 節點地址列表（例如 `["worker1:9000", "worker2:9000"]`）。 |
| `send_source` | 布尔值 | `false` | 压缩并传输源码快照至无共享存储的远程 Worker 節點。 |

---

### `[daemon]` —— 后台 IPC 守护服务

| 配置键 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `port` | 整数 | `9527` | Fish 后台守护进程所监听的本地回环 TCP 端口。 |

---

### `[pipelines.<task>]` —— 任务流水线拓撲

配置各任务之间的依赖顺序以及缓存边界：

```toml
[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "Cargo.toml"]
outputs = ["target/release/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]

[pipelines.lint]
inputs = ["src/**/*.rs"]
```
