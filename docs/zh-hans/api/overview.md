# 核心 API 与架构概览

> 🌐 **Translations & Contributions:** [Translation Guidelines](TRANSLATION.md)

Fish 核心引擎各组件与数据结构参考文档。

## 1. 工作区建模 (`fish-core`)
- `Package`: Workspace package abstraction.
- `Workspace`: Monorepo multi-package coordinator.
- `Manifest`: Project configuration model.

## 2. 有向无环图 (DAG) (`fish-graph`)
- `BuildGraph`: Directed Acyclic Graph (DAG).
- `GraphQueryEngine`: Algebraic query engine (`deps`, `rdeps`, `somepath`).

## 3. 执行引擎与沙箱 (`fish-executor`)
- `Executor`: Safe process runner with sandbox isolation.

## 4. 高并发任务调度器 (`fish-scheduler`)
- `Scheduler`: GNU Jobserver integration and parallel task dispatcher.

## 5. CAS 内容寻址缓存 (`fish-cache` & `fish-cas`)
- `LocalCache`: Two-phase pruning fingerprint cache.
- `CasStorage`: Content-addressable storage with ZSTD compression.

## 6. Python AI 智能服务 (`py/fish_ai`)
- `FailureAnalyzer`: AI failure diagnostics.
- `ScheduleOptimizer`: Critical-path scheduling optimizer.

## 7. Go 云原生网络与协调服务 (`go/pkg`)
- `NodeRegistry`: Distributed worker coordinator.
- `LoadBalancer`: Least-loaded worker proxy.
