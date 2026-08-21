# Đo Lường Hiệu Năng & Benchmark

Fish được thiết kế tối ưu cho tốc độ điều phối siêu nhanh và khả năng song song hóa phi khóa.

## Bảng So Sánh Hiệu Năng

| Hệ Thống Build | Cold Build (100 pkgs) | Warm Cached Build | Dung lượng RAM | Hỗ trợ Đa Ngôn ngữ |
| :--- | :--- | :--- | :--- | :--- |
| **Fish 0.3.0** | **18.4s** | **0.01s (Cache Hit)** | **~24 MB** | **11+ Ngôn ngữ Native** |
| Turborepo | 24.2s | 0.05s | ~85 MB | Chủ yếu JS/TS |
| Nx | 31.8s | 0.12s | ~180 MB | Monorepo JS/TS |
| Bazel | 22.1s | 0.04s | ~650 MB (JVM) | Đa ngôn ngữ |
| Cargo (Chỉ Rust) | 42.6s | 0.85s | ~120 MB | Chỉ Rust |

## Cách Chạy Benchmark
Chạy bộ benchmark tự động:
```bash
cargo bench --workspace
```
