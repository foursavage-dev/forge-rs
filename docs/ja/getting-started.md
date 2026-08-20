# Fish スタートガイド

> 🌐 **翻訳と貢献：** このドキュメントをご自身の言語に翻訳または改善しませんか？ [翻訳ガイドライン](TRANSLATION.md) をご覧ください。

このガイドでは、高速でキャッシュファーストな多言語ビルドオーケストレーションシステム「Fish」の基本的な使い方を解説します。

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

### ソースコードからビルド

```bash
git clone https://github.com/requla11/fish.git
cd fish
cargo install --path crates/fish-cli
```

### Cargo 経由でインストール

```bash
cargo install fish-cli --git https://github.com/requla11/fish
```

## クイックスタート

### Rust プロジェクトのビルド

```bash
cd my-rust-project
fish init
fish build
```

### 多言語プロジェクト (Polyglot) のビルド

```bash
# ワークスペース全体の高速チェック
fish check

# すべてのテストを実行
fish test

# キャッシュと生成物のクリーンアップ
fish clean
```

## インタラクティブ TUI の起動

```bash
fish ui
```

## AI 診断とスケジューリング最適化

```bash
# AI を用いたビルドエラー原因分析
fish ai analyze --toolchain rust --stderr "error[E0308]: mismatched types"

# DAG 実行順序の最適化
fish ai optimize --workers 8

# Git Diff に基づく影響パッケージの推薦
fish ai recommend
```
