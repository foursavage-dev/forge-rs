# Dotnet 語言後端支援

> 🌐 **Translations & Contributions:** [Translation Guidelines](TRANSLATION.md)

Fish 為各種主流語言專案提供原生高效的建置編排支援。

## 自動偵測

自動偵測: `*.csproj`.

## fish.toml 設定檔設定

```toml
[build]
backend = "dotnet"
jobs = 8

[pipelines.build]
inputs = ["src/**/*", "*.csproj"]
outputs = ["target/**/*"]
```

## 自動產生的建置任務

- `fish build`: 自動產生的建置任務 (build)
- `fish test`: 自動產生的建置任務 (test)
- `fish check`: 自動產生的建置任務 (check)

## 相依關係提取

- `*.csproj`
