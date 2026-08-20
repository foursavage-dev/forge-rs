# 設定リファレンス (`fish.toml`)

> 🌐 **翻訳と貢献：** このドキュメントをご自身の言語に翻訳または改善しませんか？ [翻訳ガイドライン](TRANSLATION.md) をご覧ください。

Fish はワークスペース直下の `fish.toml` ファイルで設定を管理します。

## 設定例

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

## 各設定セクションの解説

### `[workspace]`
- `name`: ワークスペースの一意な識別名。
- `members`: サブパッケージのパスを示す Glob パターン一覧。

### `[cache]`
- `enabled`: キャッシュ機能の有効/無効。
- `storage_dir`: CAS ストレージの配置ディレクトリ。
- `max_size_gb`: L1 キャッシュの最大容量（超過時は自動プルーニング）。
- `compression`: アーティファクトの圧縮方式（`zstd` または `none`）。

### `[scheduler]`
- `max_jobs`: 最大同時実行タスク数。
- `strategy`: スケジューリング戦略（`critical-path`、`fifo`、`least-loaded`）。
