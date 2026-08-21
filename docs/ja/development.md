# Fish 開発ガイド

> 🌐 **翻訳と貢献:** このドキュメントをあなたの言語で翻訳または改善したいですか？ [翻訳ガイドライン](TRANSLATION.md) をご覧ください。

このドキュメントは、Fish コードベースに貢献する開発者向けの詳細な手順を説明します。

## 前提条件

- Rust 1.88 以上 (MSRV 1.88)
- Git
- エディタ / IDE (VS Code 推奨)
- Docker (任意、コンテナ検証用)

## 環境構築

```bash
# リポジトリのクローン
git clone https://github.com/requla11/fish.git
cd fish

# CLI のビルド
cargo build -p fish-cli

# 全テストの実行
cargo test --workspace
```

## クレート構成

- `crates/fish-core`: プロジェクト検出、マニフェスト解析、コンパイルデータベース生成。
- `crates/fish-graph`: DAG ビルドグラフ、トポロジカルソート、代数クエリ。
- `crates/fish-executor`: 非同期プロセス実行、レスポンスファイル、高速 CoW クローン。
- `crates/fish-scheduler`: ワークスティーリングスケジューラ、メモリガバナー、GNU jobserver。
- `crates/fish-cache` & `fish-cas`: フィンガープリント、Zstd CAS ストレージ。
- `crates/fish-backend-*`: 11以上の言語バックエンド。
- `crates/fish-cli`: CLI インターフェイスと Web ダッシュボード。

## 品質検証

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```
