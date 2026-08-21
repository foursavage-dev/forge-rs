# Rust バックエンド

> 🌐 **翻訳と貢献:** このドキュメントをあなたの言語で翻訳または改善したいですか？ [翻訳ガイドライン](../TRANSLATION.md) をご覧ください。

Rust バックエンドは、Cargo を使用する Rust プロジェクト向けにビルドオーケストレーションとキャッシュ機能を提供します。

## 自動検出 (Detection)

プロジェクトディレクトリに `Cargo.toml` ファイルが存在する場合、Rust バックエンドが自動的に検出されます。

## 設定 (Configuration)

プロジェクトまたはワークスペースのルートにある `fish.toml` で Rust バックエンドを設定します：

```toml
[build]
backend = "rust"
jobs = 8
no_cache = false
semantic = true
critical_path = true

[pipelines.build]
inputs = ["src/**/*", "Cargo.toml", "Cargo.lock"]
outputs = ["target/release/**/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]
```

## 生成されるタスク (Tasks Generated)

### ビルドタスク (Build Task)
```bash
cargo build --release --features <features>
```

### テストタスク (Test Task)
```bash
cargo test --release --features <features>
```

### チェックタスク (Check Task)
```bash
cargo check --release --features <features>
```

### ドキュメント生成タスク (Doc Task)
```bash
cargo doc --release --features <features>
```

## 依存関係の抽出 (Dependency Extraction)

Rust バックエンドは以下から依存関係を解析します：
- `Cargo.toml` の依存関係セクション
- `Cargo.lock` の正確なバージョン
- ワークスペース内部クレート間の依存関係

## フィンガープリント計算 (Fingerprinting)

Rust バックエンドは以下に基づいてキャッシュハッシュを計算します：
- `Cargo.toml` の内容
- `Cargo.lock` の内容
- ソースファイル全体（`target/` ディレクトリは自動除外）
- ビルド設定とコンパイラフラグ

## 使用例 (Examples)

### 基本的な Rust プロジェクトのビルド
```bash
cd my-rust-project
fish build
```

### 特定の Features を指定したワークスペースのビルド
```bash
cd my-workspace
fish build -p my-package --features "serde,uuid"
```

### ワークスペース全体のテスト実行
```bash
cd my-workspace
fish test
```

## 前提条件
- システムに Rust ツールチェーン (`rustc`, `cargo`) がインストールされている必要があります。
