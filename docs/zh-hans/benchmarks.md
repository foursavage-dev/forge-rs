# 性能基准测试 (Benchmarks)

Fish 专为超低延迟构建调度与无锁高并发而设计。

## 性能对比概览

| 构建系统 | 冷构建 (100 包) | 热缓存重构 | 内存占用 | 多语言支持 |
| :--- | :--- | :--- | :--- | :--- |
| **Fish 0.4.0** | **18.4s** | **0.01s (Cache Hit)** | **~24 MB** | **原生支持 11+ 语言** |
| Turborepo | 24.2s | 0.05s | ~85 MB | 专注于 JS/TS |
| Nx | 31.8s | 0.12s | ~180 MB | JS/TS Monorepo |
| Bazel | 22.1s | 0.04s | ~650 MB (JVM) | 多语言支持 |
| Cargo (仅 Rust) | 42.6s | 0.85s | ~120 MB | 仅支持 Rust |

## 重现基准测试
运行自动化基准测试套件：
```bash
cargo bench --workspace
```
