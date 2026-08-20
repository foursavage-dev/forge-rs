# Fish スタートガイド

> 🌐 **翻訳と貢献:** このドキュメントをあなたの言語で翻訳または改善したいですか？ [翻訳ガイドライン](TRANSLATION.md) をご覧ください。

このガイドでは、高速でキャッシュ優先のビルドオーケストレーションシステムである Fish の基本操作と利用開始手順を解説します。

## インストール

### ワンラインインストール（推奨）

**Linux & macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/requla11/fish/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/requla11/fish/main/install.ps1 | iex
```

### ソースコードからのビルド

```bash
# リポジトリのクローン
git clone https://github.com/requla11/fish.git
cd fish

# ビルドとインストール
cargo install --path crates/fish-cli
```

### Cargo からのインストール

```bash
cargo install fish-cli --git https://github.com/requla11/fish
```

## クイックスタート

### Rust プロジェクトのビルド

```bash
cd your-rust-project
fish build
```

### 多言語モノレポのビルド

```bash
# サンプルモノレポをクローン
git clone https://github.com/requla11/fish.git
cd fish/examples/polyglot-demo

# すべてのサービスをビルド
fish build

# 依存グラフを表示
fish graph

# テストを実行
fish test
```

## 基本コマンド

### ビルドコマンド

```bash
# ワークスペース全体をビルド
fish build

# 特定のパッケージをビルド
fish build -p my-package

# 8スレッド並行でビルド
fish build -j 8

# キャッシュを無効化してビルド
fish build --no-cache

# サンドボックス環境でビルド
fish build --sandbox

# 再ビルド理由の詳細な説明を出力
fish build --explain

# プロファイルに基づく最適化 (PGO) ワークフロー
fish build --pgo-generate
# ... ベンチマークやワークロードを実行 ...
fish build --pgo-use
```

### グラフとクエリコマンド (Graph & Query)

```bash
# 推移的依存関係のクエリ (Bazelスタイル)
fish query "deps(//fish-cli)"

# 逆依存関係のクエリ
fish query "rdeps(//fish-graph)"

# 2つのモジュール間の全パスを検索
fish query "allpaths(//fish-cli, //fish-core)"

# 正規表現で依存関係をフィルタリング
fish query "filter('backend', deps(//fish-cli))"

# 視覚的なグラフ描画
fish graph --format tree
fish graph --format dot
```

### デーモンコマンド (Daemon)

```bash
# サブミリ秒単位のウォームビルドを実現するバックグラウンドデーモンを起動
fish daemon start

# デーモンの状態を確認
fish daemon status

# デーモンを停止
fish daemon stop
```

### テストコマンド

```bash
# すべてのテストを実行
fish test

# 特定のパッケージをテスト
fish test -p my-package

# キャッシュ無効でテストを実行
fish test --no-cache
```

### キャッシュ管理コマンド

```bash
# キャッシュ統計を表示
fish cache stats

# 古いキャッシュをクリーンアップ
fish cache prune

# リモートキャッシュサーバーを起動
fish cache-server --listen 0.0.0.0:8080
```

### 分散ビルドコマンド

```bash
# ワーカーを起動
fish worker --listen 0.0.0.0:9000

# 分散ワーカークラスタを使用してビルド
fish build --workers worker1:9000,worker2:9000
```

### CI/CD 設定生成コマンド

```bash
# GitHub Actions ワークフローの生成
fish ci init --platform github

# GitLab CI パイプラインの生成
fish ci init --platform gitlab

# CircleCI 設定の生成
fish ci init --platform circleci

# Bitbucket Pipelines 設定の生成
fish ci init --platform bitbucket

# サポートされているすべての CI 設定を生成
fish ci init --platform all
```

### プラグインコマンド

```bash
# 利用可能なプラグイン一覧
fish plugin list

# プラグインコマンドを実行
fish plugin execute my-plugin build

# プラグインをインストール
fish plugin install ./my-plugin
```

## 設定

### ワークスペース設定 (`fish.toml`)

Fish はマニフェストファイルに基づいてプロジェクトタイプを自動検出します。カスタム実行、キャッシュ、パイプラインを設定するにはプロジェクトのルートに `fish.toml` を配置します：

```toml
[build]
backend = "auto"
jobs = 8
no_cache = false
sandbox = false
semantic = true
critical_path = true
ram_limit = 85

[cache]
dir = "~/.fish/cache"
reflink = true

[remote]
cache_url = "http://127.0.0.1:8080"
token = "secret-cache-token"

[daemon]
port = 9527

[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "Cargo.toml"]
outputs = ["target/release/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]
```

完全な設定オプションは [設定ガイド](configuration.md) を参照してください。

---

## インタラクティブテレメトリ & Web ダッシュボード

Fish には、5言語（英語、ベトナム語、簡体字中国語、繁体字中国語、日本語）に対応したリアルタイムのインタラクティブ DAG ビジュアライザとテレメトリダッシュボードが組み込まれています：

```bash
# ポート3000でWebダッシュボードを起動し、ブラウザで自動表示
fish ui --port 3000 --open

# JSONグラフデータの取得
curl http://localhost:3000/api/graph

# ハードウェアと CAS 統計の確認
curl http://localhost:3000/api/stats
```

---

## トラブルシューティング

### ビルドの失敗

ビルドが失敗した場合：

1. エラーメッセージを確認するか `fish build --explain` で再ビルド理由を診断してください。
2. デバッグログ付きで実行: `RUST_LOG=debug fish build`
3. ツールチェーンの状態を確認: `fish doctor`
4. キャッシュをクリア: `fish cache prune`

### キャッシュの問題

キャッシュが機能しない場合：

1. キャッシュ統計を確認: `fish cache stats`
2. キャッシュディレクトリの書き込み権限を確認: `~/.fish/cache`
3. キャッシュをクリアして再ビルド: `fish cache prune && fish build`

### ワーカー接続エラー

分散ワーカーに接続できない場合：

1. ネットワーク接続を確認
2. ワーカーが稼働しているか確認: `fish worker --listen 0.0.0.0:9000`
3. ファイアウォール設定と認証トークンを確認
4. ワーカーのログ出力を確認

## 次のステップ

- [アーキテクチャガイド](architecture.md) を読む
- [開発ガイド](../development.md) を確認
- [CLI コマンドリファレンス](cli-reference.md) を参照
- [言語別バックエンドドキュメント](backends/) を確認

## サポートと問い合わせ

- [公式ドキュメント](../getting-started.md)
- [サポート窓口](../support.md)
- [GitHub Issues](https://github.com/requla11/fish/issues)
- [連絡先メール](mailto:foursavage@proton.me)
