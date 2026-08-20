# Rust バックエンド

> 🌐 **Translations & Contributions:** [Translation Guidelines](TRANSLATION.md)

Fish は主要な各プログラミング言語プロジェクトに対して高速なビルドオーケストレーションを提供します。

## プロジェクトの自動検出

プロジェクトの自動検出: `Cargo.toml`.

## fish.toml での設定

```toml
[build]
backend = "rust"
jobs = 8

[pipelines.build]
inputs = ["src/**/*", "Cargo.toml"]
outputs = ["target/**/*"]
```

## 自動生成されるタスク

- `fish build`: 自動生成されるタスク (build)
- `fish test`: 自動生成されるタスク (test)
- `fish check`: 自動生成されるタスク (check)

## 依存関係の抽出

- `Cargo.toml`
