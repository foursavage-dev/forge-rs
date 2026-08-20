# 常见问题与故障排查

> 🌐 **翻译与贡献：** 想用您的母语翻译或完善本文档？请查看 [翻译指南](TRANSLATION.md)。

## 常见问题

### 1. Fish 与 Cargo、Turborepo 或 Bazel 有何区别？
Fish 专为多语言大型单体仓库打造，兼备 Rust 原生极致执行效率、Python AI 智能优化以及 Go 云原生分布式网络，无需 Bazel 复杂的规则配置即可开箱即用。

### 2. Fish 支持哪些后端语言？
目前 Fish 官方支持 11 种主流语言与工具链：Rust、Go、TypeScript/Node.js、Python、C/C++、Docker、Java、.NET、Swift、Dart 以及 Zig。

### 3. 如何检测当前机器的开发工具链？
运行以下命令即可：
```bash
fish doctor --ai
```

## 故障排查

### Windows 虚拟内存耗尽报错 (`os error 1455`)
- **原因:** 并发编译过多大型宏或重型依赖占满分页文件。
- **解决方案:** 通过 `--jobs` 限制并发数：
```bash
fish build --jobs 4
```
