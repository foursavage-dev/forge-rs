# Fish AI Agent 开发工作流指南

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](TRANSLATION.md)。

本文档为参与 Fish 构建编排系统开发的人工智能编码代理（AI Coding Agents）提供详细的端到端操作规范。

## 🎯 概述

本工作流旨在：
- 最大限度减少 Bug 和潜在错误的引入。
- 确保系统各模块平稳运行。
- 保持高质量的代码规范与架构一致性。
- 严格遵循项目特定的工程最佳实践。

---

## 📖 阶段一：前期上下文收集

### 步骤 1.1：阅读核心文档（强制要求）

**建议阅读顺序：**
1. **README.md** - 项目概览、快速入门与基础命令。
2. **Cargo.toml** - 工作区结构、依赖项及最低 Rust 版本 (1.88+)。
3. **ARCHITECTURE.md** - 架构设计与各组件职责。
4. **DEVELOPMENT.md** - 本地开发环境搭建与工作流。

### 步骤 1.2：根据具体任务阅读模块文档

| 任务类型 | 需额外阅读的文件 |
|---|---|
| 语言后端开发 | `crates/fish-backend-rust/` (作为参考), `ARCHITECTURE.md` 后端部分 |
| 调度器逻辑修改 | `crates/fish-scheduler/` 源码 |
| 缓存与 CAS 改进 | `crates/fish-cache/` 与 `crates/fish-cas/` 源码 |
| CLI 命令拓展 | `crates/fish-cli/` 源码 |
| 安全审计与签名 | `crates/fish-security/` 与 `crates/fish-signing/` |

---

## 🎯 阶段二：任务分析与规划

在动手编码前明确以下问题：
- 本次修改解决的具体问题是什么？
- 涉及哪些 Crate 和公共 API？
- 是否有现有的模式可以复用？
- 如何设计对应的自动化测试用例？

---

## 💻 阶段三：编码实现规范

- 采用 Rust 2024 Edition 规范。
- 错误处理：应用层使用 `anyhow`，库模块使用 `thiserror`。
- 异步编程：合理运用 `async/await` 与 Tokio 运行时。
- **所有函数名、变量名、类型及 Commit 记录必须统一使用英文**。

---

## 🔍 阶段四：验证与质量检查

```bash
# 格式化校验
cargo fmt --all -- --check

# Clippy 静态代码检查
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 运行完整测试套件
cargo test --workspace
```
