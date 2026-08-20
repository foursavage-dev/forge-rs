# Go 語言後端

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](../TRANSLATION.md)。

Go 后端为使用 Go 语言开发的服务与工具提供构建编排与缓存加速。

## 自动检测 (Detection)
当项目目录中存在 `go.mod` 文件时自动启用。

## 项目配置 (`fish.toml`)
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

## 自动生成的任务
- **构建任务**: `go build -o <output> ./...`
- **测试任务**: `go test -v ./...`
- **代码审查**: `go vet ./...`
