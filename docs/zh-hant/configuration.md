# 設定指南 (`fish.toml`)

> 🌐 **翻譯與貢獻：** 想用您的母語翻譯或完善本文件？請參閱 [翻譯指南](TRANSLATION.md)。

Fish 使用 `fish.toml` 檔案進行設定管理。

## 完整範例

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

## 設定項目詳解

### `[workspace]`
- `name`: 工作區唯一識別名稱。
- `members`: 包含的子專案與套件路徑 Glob 模式。

### `[cache]`
- `enabled`: 是否啟用建置快取。
- `storage_dir`: CAS 儲存目錄路徑。
- `max_size_gb`: 本機快取上限（超額將觸發兩階段清理）。
- `compression`: 產生物壓縮演算法（`zstd` 或 `none`）。

### `[scheduler]`
- `max_jobs`: 最大並行建置任務數。
- `strategy`: 排程演算法策略（`critical-path`、`fifo`、`least-loaded`）。
