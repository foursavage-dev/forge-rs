# 移行ガイド：Fish への移行

このガイドでは、Turborepo、Nx、Bazel の既存ビルド設定を統一 `fish.toml` に移行する手順を説明します。

---

## 1. Turborepo (`turbo.json`) からの移行

Turborepo のパイプライン設定は、Fish の `[pipelines]` 定義に直接対応します：

### 移行前：`turbo.json`
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

### 移行後：`fish.toml`
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

## 2. 自動初期化と検証
```bash
fish init --force
fish doctor --fix
```
