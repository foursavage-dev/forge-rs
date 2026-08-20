# 迁移指南：平滑迁移至 Fish

本指南详细介绍了如何将现有的 Turborepo、Nx 以及 Bazel 构建配置平滑迁移至统一的 `fish.toml`。

---

## 1. 从 Turborepo (`turbo.json`) 迁移

Turborepo 的 pipeline 任务依赖与产物配置可直接映射到 Fish 的 `[pipelines]` 定义中：

### 迁移前：`turbo.json`
```json
{
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": ["dist/**"]
    }
  }
}
```

### 迁移后：`fish.toml`
```toml
[build]
backend = "ts"
jobs = 8
reflink = true
semantic = true

[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "package.json"]
outputs = ["dist/**"]
```

---

## 2. 自动迁移工具
使用 Fish 内置的自动分析与修复命令快速完成 Monorepo 转换：
```bash
fish init --force
fish doctor --fix
```
