# 遠端執行與分散式快取

Fish 支援高效能分散式快取與遠端計算執行協定。

## Remote Execution API (REAPI v2)
Fish 支援基於 gRPC 的 Google / Bazel REAPI v2 協定：
```bash
fish build --remote-workers 10.0.0.1:8980,10.0.0.2:8980
```

## P2P Swarm 區域網路快取
支援區域網路內零設定點對點產物共享：
```bash
fish build --swarm
```

## 雲端 CAS 儲存
在 `fish.toml` 中設定 S3、GCS、Azure Blob 或 Redis：
```toml
[cache]
enabled = true
remote_url = "s3://my-fish-cache-bucket"
compression = "zstd"
```
