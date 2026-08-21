# Fish アーキテクチャガイド

> 🌐 **翻訳と貢献:** このドキュメントをあなたの言語で翻訳または改善したいですか？ [翻訳ガイドライン](TRANSLATION.md) をご覧ください。

このドキュメントでは、Fish のシステムアーキテクチャ、コアエンジンモジュール、および実行パイプラインに関する包括的な技術概要を説明します。

---

## システム概要

Fish は、多言語モノレポおよび分散開発環境向けに設計された、高性能でキャッシュ優先のビルドオーケストレーションシステムです。ネイティブコンパイラを置き換えるのではなく、依存関係 DAG、コンテンツアドレス可能キャッシュ (CAS)、密閉サンドボックス、並列ワークスティーリング実行をインテリジェントに統合管理するオーケストレーション層として機能します。

```text
┌─────────────────────────────────────────────────────────────┐
│                    fish-cli / Web UI                        │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│       fish-core (Discovery, Toolchains, compile_commands)   │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│           fish-graph (DAG & Algebraic Query Engine)         │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│   fish-scheduler (Governor, Jobserver, Racing, Watcher)     │
└──────────────┬──────────────────────────────┬───────────────┘
               │                              │
┌──────────────▼──────────────┐┌──────────────▼──────────────┐
│ fish-executor & Middleware  ││  fish-cache & fish-cas      │
└──────────────┬──────────────┘└──────────────┬──────────────┘
               │                              │
┌──────────────▼──────────────────────────────▼──────────────┐
│      11+ Language Backends & Distributed Worker Network     │
└─────────────────────────────────────────────────────────────┘
```

---

## コアクレートと責務

### 1. ワークスペース検出 (`fish-core`)
- **マニフェスト検出**: `Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, `CMakeLists.txt`, `pom.xml`, `*.csproj`, `Package.swift`, `pubspec.yaml`, `build.zig`, `Dockerfile` をスキャンして解析。
- **コンパイルデータベース生成**: Clangd や IDE 向けの標準 `compile_commands.json` を生成 (`CompilationDatabase`)。
- **密閉ツールチェーン管理**: コンパイラのバイナリパスと環境変数を隔離して管理 (`ToolchainRegistry`, `ToolchainSpec`)。
- **マイクロ入力フィルタリング**: Glob パターンに基づいて入力ファイルを正確に絞り込み、不要なキャッシュ無効化を防止 (`MicroInputFilter`)。

### 2. ビルドグラフ (`fish-graph`)
- **トポロジカルタスクグラフ**: ビルドタスクの有向非巡回グラフ (DAG) を構築し、循環依存を検出。
- **代数グラフクエリ**: Bazel スタイルのグラフ式 (`deps()`, `rdeps()`, `allpaths()`, `somepath()`, `filter()`) を評価。
- **動的ノード展開**: 実行中にサブタスクグラフを動的に生成 (`DynamicGraphExpander`)。

### 3. 実行と高速実体化 (`fish-executor`)
- **プロセスのオーケストレーション**: タイムアウトとストリームキャプチャを備えた非同期タスク実行。
- **高速エクステントクローニング (Fast Extents Cloning)**: Copy-on-Write (CoW) とハードリンクを利用して I/O コピーなしでアーティファクトを実体化 (`KernelCowCloner`)。
- **リンカーディスパッチャ**: `mold`, `lld`, `lld-link`, `msvc` のリンカー引数を自動検出・最適化 (`LinkerDispatcher`)。
- **コンパイラレスポンスファイル**: コマンド引数が OS の上限を超える場合に `@fish_args.rsp` を自動生成。

### 4. スケジューラとリソース制御 (`fish-scheduler`)
- **並列ワークスティーリング**: 利用可能な全ハードウェアコアでロックフリーなタスクスケジューリングを実行。
- **カーネルリソースガバナー**: メモリ使用状況をリアルタイムで監視し、OOM クラッシュを防ぐために並列数を動的に制御 (`KernelResourceGovernor`)。
- **パイプライン化コンパイル**: メタデータが準備できた時点で下流のタスクを即座にアンブロック (`PipelinedCompilationCoordinator`)。
- **GNU Jobserver プール**: ネストされたコンパイラ呼び出し間でスレッドトークン割り当てをグローバルに管理 (`JobserverPool`)。
- **動的リモートレーシング**: ローカル実行と分散クラスタワーカーを競争させ、最速の結果を採用 (`DynamicRacingExecutor`)。
- **分散タスク実行 (DTE)**: 最長処理時間 (LPT) ビンパッキングアルゴリズムによる CI 負荷分散 (`DteBinPacker`)。
- **リアルタイムファイル監視**: バックグラウンドデーモンがファイル変更イベントを監視し、キャッシュグラフを事前にウォームアップ (`FsWatcherDaemon`)。

### 5. コンテンツアドレス可能ストレージ (`fish-cache` & `fish-cas`)
- **フィンガープリント**: ソースファイル、環境変数、コンパイラフラグに対して Blake3 ハッシュを計算。
- **CAS ストレージ**: Zstandard 高速圧縮による重複排除された成果物ストレージ。
- **階層型複合キャッシュ**: L1 ローカルインメモリ/ディスクキャッシュと L2 リモート S3/HTTP キャッシュの統合。

### 6. ユーザーインターフェイス & テレメトリ (`fish-cli`)
- **コマンドラインインターフェイス**: build, test, check, graph, doctor, query, affected, daemon コマンドを提供。
- **インタラクティブ SVG DAG ビジュアライザ**: パン/ズーム、検索、ノードフォーカス、クリティカルパス強調表示を備えた Web ベースのリアルタイムキャンバス。
- **5言語 UI ローカライゼーション**: 英語、ベトナム語、簡体字中国語、繁体字中国語、日本語をサポート。
- **バックグラウンドデーモン IPC**: 即時のウォームグラフ解決を実現する `127.0.0.1:9527` ループバック TCP サービス。

---

## サポート言語バックエンド

Fish には 11 種類の専用言語アダプターが含まれています：

| バックエンド | 識別子 | 主要マニフェスト | デフォルトコンパイラ / ツール |
| :--- | :--- | :--- | :--- |
| **Rust** | `rust` | `Cargo.toml` | `cargo`, `rustc` |
| **C / C++** | `cc` | `CMakeLists.txt`, `Makefile` | `cmake`, `clang`, `gcc`, `msvc` |
| **Go** | `go` | `go.mod` | `go build`, `go test` |
| **TypeScript / Node** | `ts` | `package.json` | `npm`, `pnpm`, `yarn`, `bun` |
| **Python** | `py` | `pyproject.toml`, `requirements.txt` | `python -m build`, `pytest`, `uv` |
| **Java / Kotlin** | `java` | `pom.xml`, `build.gradle` | `mvn`, `gradle` |
| **.NET** | `dotnet` | `*.csproj`, `*.sln` | `dotnet build`, `dotnet test` |
| **Swift** | `swift` | `Package.swift` | `swift build`, `swift test` |
| **Dart / Flutter** | `dart` | `pubspec.yaml` | `dart compile`, `flutter build` |
| **Zig** | `zig` | `build.zig` | `zig build` |
| **Docker** | `docker` | `Dockerfile` | `docker build` |

---

## セキュリティと成果物検証

- **暗号署名 (`fish-signing`)**: Ed25519 アルゴリズムによるデジタル署名の生成と検証。
- **SBOM 生成**: SPDX および CycloneDX 形式のソフトウェア部品表エクスポート。
- **脆弱性スキャナー (`fish-security`)**: CVSS スコアリングに基づく依存関係の脆弱性検出と重大度によるビルド制御。
- **シークレット管理 (`fish-secrets`)**: HashiCorp Vault、AWS Secrets Manager、Kubernetes Secret との統合およびコンソールログの自動マスキング。
