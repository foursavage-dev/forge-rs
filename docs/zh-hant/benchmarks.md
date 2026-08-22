# 效能基準測試 (Benchmarks)

Fish 專為超低延遲建置調度與無鎖高並發而設計。

## 效能對比概覽

| 建置系統 | 冷建置 (100 包) | 熱快取重構 | 記憶體佔用 | 多語言支援 |
| :--- | :--- | :--- | :--- | :--- |
| **Fish 0.4.0** | **18.4s** | **0.01s (Cache Hit)** | **~24 MB** | **原生支援 11+ 語言** |
| Turborepo | 24.2s | 0.05s | ~85 MB | 專注於 JS/TS |
| Nx | 31.8s | 0.12s | ~180 MB | JS/TS Monorepo |
| Bazel | 22.1s | 0.04s | ~650 MB (JVM) | 多語言支援 |
| Cargo (僅 Rust) | 42.6s | 0.85s | ~120 MB | 僅支援 Rust |

## 重現基準測試
執行自動化基準測試套件：
```bash
cargo bench --workspace
```
