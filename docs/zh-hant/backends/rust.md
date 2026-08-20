# Rust 語言後端

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](../TRANSLATION.md)。

Rust 后端为使用 Cargo 的 Rust 项目提供专业的構建编排与快取加速支持。

## 自动检测 (Detection)

当项目目录中存在 `Cargo.toml` 文件时，Fish 会自动启用 Rust 后端。

## 项目配置 (Configuration)

在项目或工作區根目录的 `fish.toml` 中配置 Rust 后端：

```toml
[build]
backend = "rust"
jobs = 8
no_cache = false
semantic = true
critical_path = true

[pipelines.build]
inputs = ["src/**/*", "Cargo.toml", "Cargo.lock"]
outputs = ["target/release/**/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]
```

## 自动生成的任务 (Tasks Generated)

### 構建任务 (Build Task)
```bash
cargo build --release --features <features>
```

### 测试任务 (Test Task)
```bash
cargo test --release --features <features>
```

### 快速检查任务 (Check Task)
```bash
cargo check --release --features <features>
```

### 文档生成任务 (Doc Task)
```bash
cargo doc --release --features <features>
```

## 依赖解析 (Dependency Extraction)

Rust 后端从以下位置解析依赖图关系：
- `Cargo.toml` 的依赖段落
- `Cargo.lock` 精确版本锁定文件
- 工作區内部 Crate 间的相互依赖

## 指纹计算 (Fingerprinting)

Rust 后端基于以下内容计算快取唯一哈希：
- `Cargo.toml` 文件内容
- `Cargo.lock` 文件内容
- 所有源码文件（自动排除 `target/` 目录）
- 编译标志与环境变量

## 使用示例 (Examples)

### 基础 Rust 项目構建
```bash
cd my-rust-project
fish build
```

### 带有特定 Features 的工作區構建
```bash
cd my-workspace
fish build -p my-package --features "serde,uuid"
```

### 运行工作區测试
```bash
cd my-workspace
fish test
```

## 前置要求
- 系统环境中需安装 Rust 工具链 (`rustc`, `cargo`)。
