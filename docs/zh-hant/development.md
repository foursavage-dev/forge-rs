# Fish 開發指南

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](TRANSLATION.md)。

本指南为参与 Fish 代码库开发的工程师提供完整的开发流程指导。

## 前置条件

- Rust 1.88 或更高版本 (MSRV 1.88)
- Git
- 文本编辑器 / IDE (推荐 VS Code)
- Docker (可选，用于测试容器化任务)

## 环境搭建

```bash
# 克隆代码仓库
git clone https://github.com/requla11/fish.git
cd fish

# 编译 CLI 工具
cargo build -p fish-cli

# 运行全工作區测试
cargo test --workspace
```

## 模块结构划分

- `crates/fish-core`: 项目发现、清单解析、编译数据库生成。
- `crates/fish-graph`: 依赖 DAG 图构建、拓扑排序、代数图查询。
- `crates/fish-executor`: 异步进程执行、响应参数文件、快速 CoW 克隆。
- `crates/fish-scheduler`: 工作窃取调度器、内核资源管控器、GNU Jobserver。
- `crates/fish-cache` & `fish-cas`: 指纹计算、Zstd 内容寻址存储。
- `crates/fish-backend-*`: 支持 11+ 种主流语言的后端适配器。
- `crates/fish-cli`: 命令行工具及交互式 Web 仪表板。

## 质量验证

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```
