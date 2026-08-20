# CLI コマンドリファレンス

> 🌐 **翻訳と貢献：** このドキュメントをご自身の言語に翻訳または改善しませんか？ [翻訳ガイドライン](TRANSLATION.md) をご覧ください。

Fish CLI の全コマンドとオプション一覧です。

## 基本コマンド

| コマンド | 説明 |
| :--- | :--- |
| `fish init` | 現在のディレクトリに `fish.toml` を初期化 |
| `fish new <name>` | テンプレートから新しいプロジェクトを作成 |
| `fish build` | ワークスペース全体のタスクをビルド |
| `fish check` | 構文と型の高速チェックを実行 |
| `fish test` | テストスイートを並行実行 |
| `fish clean` | 生成物とローカルキャッシュを削除 |

## AI コマンド

```bash
# エラーログの分析
fish ai analyze --toolchain rust --stderr "<log_content>"

# タスクスケジュールの最適化
fish ai optimize --workers 8

# ビルド対象パッケージの推薦
fish ai recommend
```

## ネットワーク・分散コマンド

```bash
# リモートキャッシュサーバの起動
fish cache-server --listen 0.0.0.0:8080

# 分散ワーカーノードの起動
fish worker --coordinator http://coordinator:9090
```

## 診断・解析コマンド

```bash
# ツールチェーンと環境のヘルスチェック
fish doctor --ai

# 依存関係グラフのクエリ
fish query "deps(//packages/core)"

# ファイル変更の監視と自動ビルド
fish watch
```
