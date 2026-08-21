# 36 Crates アーキテクチャ (`crates/`)

Fish は 36 個の高機能 Rust クレートで構成され、階層ごとに明確に分離されています。

## アーキテクチャ階層
1. **基盤層 (Foundation Tier)**:
   - `fish-core`: マニフェストモデル、設定管理、プロジェクト検出。
   - `fish-graph`: DAG モデル、ロックフリーなトポロジカルソート、グラフクエリ。
   - `fish-executor`: OS プロセス管理、レスポンスファイル、ミドルウェアチェーン。
2. **ストレージ & キャッシュ層 (Storage Tier)**:
   - `fish-cas`: ZSTD 圧縮と FastCDC チャンキングを備えた CAS ストレージ。
   - `fish-cache`: 2 フェーズ Fingerprint キャッシュと GC。
   - `fish-remote-cache`: gRPC REAPI v2 クライアントと TCP ストリーミングキャッシュ。
3. **スケジューリング層 (Scheduling Tier)**:
   - `fish-scheduler`: クリティカルパス予測スケジューラ、ワークスティーリング、GNU Jobserver。
   - `fish-worker`: リモートワーカークラスター実行と Daemon IPC。
   - `fish-sandbox`: Linux eBPF トレースと WASM サンドボックス。
4. **11 言語バックエンド**:
   - Rust, C++, Go, TS, Python, Docker, Java, .NET, Swift, Dart, Zig。
5. **セキュリティ & 診断層**:
   - `fish-security`, `fish-signing`, `fish-secrets`, `fish-flaky-detection`, `fish-notifications`, `fish-analytics`, `fish-templates`, `fish-docker-builder`, `fish-incremental`, `fish-multiplatform`, `fish-installer`。
6. **CLI アプリケーション**:
   - `fish-cli`: 統合 CLI インターフェース、TUI リアルタイムダッシュボード、`fish lsp`。
