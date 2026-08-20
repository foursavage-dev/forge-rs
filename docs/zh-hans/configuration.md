# 配置指南 (`fish.toml`)

> 🌐 **翻译与贡献：** 想用您的母语翻译或完善本文档？请查看 [翻译指南](TRANSLATION.md)。

Fish 使用 `fish.toml` 文件进行配置管理。

## 完整示例

```toml
[workspace]
name = "my-monorepo"
members = [
    "packages/*",
    "apps/*"
]

[cache]
enabled = true
storage_dir = "~/.fish/cache"
max_size_gb = 50
compression = "zstd"

[scheduler]
max_jobs = 8
memory_limit_mb = 8192
strategy = "critical-path"

[ai]
enabled = true
endpoint = "stdio"
auto_suggest = true
```

## 配置项详解

### `[workspace]`
- `name`: 工作区唯一标识名称。
- `members`: 包含的子项目与包路径 Glob 模式。

### `[cache]`
- `enabled`: 是否启用构建缓存。
- `storage_dir`: CAS 存储目录路径。
- `max_size_gb`: 本地缓存上限（超额将触发两阶段清理）。
- `compression`: 产物压缩算法（`zstd` 或 `none`）。

### `[scheduler]`
- `max_jobs`: 最大并发构建任务数。
- `strategy`: 调度算法策略（`critical-path`、`fifo`、`least-loaded`）。
