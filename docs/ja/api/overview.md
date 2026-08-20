# コア API 概要

> 🌐 **Translations & Contributions:** [Translation Guidelines](TRANSLATION.md)

Fish コアエンジンのアーキテクチャコンポーネントおよびデータ構造リファレンス。

## 1. ワークスペースモデル (`fish-core`)
- `Package`: Workspace package abstraction.
- `Workspace`: Monorepo multi-package coordinator.
- `Manifest`: Project configuration model.

## 2. 依存グラフ (DAG) (`fish-graph`)
- `BuildGraph`: Directed Acyclic Graph (DAG).
- `GraphQueryEngine`: Algebraic query engine (`deps`, `rdeps`, `somepath`).

## 3. 実行エンジンとサンドボックス (`fish-executor`)
- `Executor`: Safe process runner with sandbox isolation.

## 4. 並行タスクスケジューラ (`fish-scheduler`)
- `Scheduler`: GNU Jobserver integration and parallel task dispatcher.

## 5. CAS コンテンツアドレス指定キャッシュ (`fish-cache` & `fish-cas`)
- `LocalCache`: Two-phase pruning fingerprint cache.
- `CasStorage`: Content-addressable storage with ZSTD compression.

## 6. Python AI インテリジェンス層 (`py/fish_ai`)
- `FailureAnalyzer`: AI failure diagnostics.
- `ScheduleOptimizer`: Critical-path scheduling optimizer.

## 7. Go クラウドネットワーク調整サービス (`go/pkg`)
- `NodeRegistry`: Distributed worker coordinator.
- `LoadBalancer`: Least-loaded worker proxy.
