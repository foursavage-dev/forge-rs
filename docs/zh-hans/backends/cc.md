# C / C++ 语言后端支持

> 🌐 **Translations & Contributions:** [Translation Guidelines](TRANSLATION.md)

Fish 为各种主流语言项目提供原生高效的构建编排支持。

## 自动检测

自动检测: `CMakeLists.txt`.

## fish.toml 配置文件设置

```toml
[build]
backend = "c / c++"
jobs = 8

[pipelines.build]
inputs = ["src/**/*", "CMakeLists.txt"]
outputs = ["target/**/*"]
```

## 自动生成的构建任务

- `fish build`: 自动生成的构建任务 (build)
- `fish test`: 自动生成的构建任务 (test)
- `fish check`: 自动生成的构建任务 (check)

## 依赖关系提取

- `CMakeLists.txt`
