# Rust 語言後端支援

> 🌐 **Translations & Contributions:** [Translation Guidelines](TRANSLATION.md)

Fish 為各種主流語言專案提供原生高效的建置編排支援。

## 自動偵測

自動偵測: `Cargo.toml`.

## fish.toml 設定檔設定

```toml
[build]
backend = "rust"
jobs = 8

[pipelines.build]
inputs = ["src/**/*", "Cargo.toml"]
outputs = ["target/**/*"]
```

## 自動產生的建置任務

- `fish build`: 自動產生的建置任務 (build)
- `fish test`: 自動產生的建置任務 (test)
- `fish check`: 自動產生的建置任務 (check)

## 相依關係提取

- `Cargo.toml`
