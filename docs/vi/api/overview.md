# Tổng quan API Lõi

> 🌐 **Translations & Contributions:** [Translation Guidelines](TRANSLATION.md)

Tài liệu tham khảo các thành phần kiến trúc và cấu trúc dữ liệu cốt lõi của Fish.

## 1. Mô hình Workspace (`fish-core`)
- `Package`: Workspace package abstraction.
- `Workspace`: Monorepo multi-package coordinator.
- `Manifest`: Project configuration model.

## 2. Đồ thị phụ thuộc DAG (`fish-graph`)
- `BuildGraph`: Directed Acyclic Graph (DAG).
- `GraphQueryEngine`: Algebraic query engine (`deps`, `rdeps`, `somepath`).

## 3. Bộ máy thực thi & Sandbox (`fish-executor`)
- `Executor`: Safe process runner with sandbox isolation.

## 4. Bộ điều phối song song (`fish-scheduler`)
- `Scheduler`: GNU Jobserver integration and parallel task dispatcher.

## 5. Hệ thống Lưu trữ & Cache CAS (`fish-cache` & `fish-cas`)
- `LocalCache`: Two-phase pruning fingerprint cache.
- `CasStorage`: Content-addressable storage with ZSTD compression.

## 6. Dịch vụ AI Phân tích & Tối ưu (`py/fish_ai`)
- `FailureAnalyzer`: AI failure diagnostics.
- `ScheduleOptimizer`: Critical-path scheduling optimizer.

## 7. Dịch vụ Mạng & Điều phối Phân tán (`go/pkg`)
- `NodeRegistry`: Distributed worker coordinator.
- `LoadBalancer`: Least-loaded worker proxy.
