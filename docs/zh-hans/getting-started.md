# 快速开始使用 Fish

> 🌐 **翻译与贡献：** 想用您的母语翻译或完善本文档？请查看 [翻译指南](TRANSLATION.md)。

本指南将帮助您快速上手 Fish —— 一款高性能、缓存优先的通用多语言构建编排系统。

## 安装指南

### 单行命令快速安装（推荐）

**Linux 与 macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/requla11/fish/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/requla11/fish/main/install.ps1 | iex
```

### 从源码编译安装

```bash
git clone https://github.com/requla11/fish.git
cd fish
cargo install --path crates/fish-cli
```

### 通过 Cargo 安装

```bash
cargo install fish-cli --git https://github.com/requla11/fish
```

## 快速入门

### 构建 Rust 项目

```bash
cd my-rust-project
fish init
fish build
```

### 构建多语言项目 (Polyglot)

```bash
# 全局快速检查
fish check

# 运行所有测试套件
fish test

# 清理构建缓存与生成文件
fish clean
```

## 体验 TUI 交互界面

```bash
fish ui
```

## AI 故障诊断与调度优化

```bash
# 使用 AI 分析构建失败原因
fish ai analyze --toolchain rust --stderr "error[E0308]: mismatched types"

# 优化 DAG 调度顺序
fish ai optimize --workers 8

# 基于 Git Diff 智能推荐构建目标
fish ai recommend
```
