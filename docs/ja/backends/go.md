# Go バックエンド

> 🌐 **翻訳と貢献:** このドキュメントをあなたの言語で翻訳または改善したいですか？ [翻訳ガイドライン](../TRANSLATION.md) をご覧ください。

Go バックエンドは、Go 言語で書かれたサービスやツール向けのビルドオーケストレーションを提供します。

## 自動検出 (Detection)
プロジェクトディレクトリに `go.mod` が存在する場合に自動検出されます。

## 設定 (`fish.toml`)
```toml
[build]
backend = "go"
jobs = 8

[pipelines.build]
inputs = ["**/*.go", "go.mod", "go.sum"]
outputs = ["bin/*"]

[pipelines.test]
inputs = ["**/*.go"]
```

## 生成されるタスク
- **ビルド**: `go build -o <output> ./...`
- **テスト**: `go test -v ./...`
- **コード検査**: `go vet ./...`
