# 核心 API 與架構概覽

> 🌐 **Translations & Contributions:** [Translation Guidelines](TRANSLATION.md)

Fish 核心引擎各元件與資料結構參考文件。

## 1. 工作區建模 (`fish-core`)
- `Package`: Workspace package abstraction.
- `Workspace`: Monorepo multi-package coordinator.
- `Manifest`: Project configuration model.

## 2. 有向無環圖 (DAG) (`fish-graph`)
- `BuildGraph`: Directed Acyclic Graph (DAG).
- `GraphQueryEngine`: Algebraic query engine (`deps`, `rdeps`, `somepath`).

## 3. 執行引擎與沙箱 (`fish-executor`)
- `Executor`: Safe process runner with sandbox isolation.

## 4. 高並行任務排程器 (`fish-scheduler`)
- `Scheduler`: GNU Jobserver integration and parallel task dispatcher.

## 5. CAS 內容定址快取 (`fish-cache` & `fish-cas`)
- `LocalCache`: Two-phase pruning fingerprint cache.
- `CasStorage`: Content-addressable storage with ZSTD compression.

## 6. Python AI 智慧服務 (`py/fish_ai`)
- `FailureAnalyzer`: AI failure diagnostics.
- `ScheduleOptimizer`: Critical-path scheduling optimizer.

## 7. Go 雲原生網路與協調服務 (`go/pkg`)
- `NodeRegistry`: Distributed worker coordinator.
- `LoadBalancer`: Least-loaded worker proxy.
