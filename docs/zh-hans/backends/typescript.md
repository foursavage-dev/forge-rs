# TypeScript / JavaScript 后端

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](../TRANSLATION.md)。

TypeScript/JavaScript 后端为 Node.js、Web 以及全栈前端项目提供构建编排支持。

## 自动检测 (Detection)

当项目目录中存在 `package.json` 时自动启用。

## 项目配置 (Configuration)

在 `fish.toml` 中配置：

```toml
[build]
backend = "ts"
jobs = 8

[pipelines.build]
inputs = ["src/**/*.{ts,tsx,js,jsx}", "package.json", "tsconfig.json"]
outputs = ["dist/**/*", "build/**/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*.{ts,js}", "src/**/*.{ts,js}"]
```

## 支持的包管理器 (Package Managers)
- **npm**: 默认的 Node.js 包管理器
- **pnpm**: 高性能且节省磁盘空间的包管理器
- **yarn**: 支持 Workspace 的依赖管理工具
- **bun**: 极速 JavaScript 运行时与包管理器

## 自动生成的任务 (Tasks Generated)

### 构建任务 (Build Task)
```bash
npm run build # 或 pnpm / yarn / bun run build
```

### 测试任务 (Test Task)
```bash
npm test # 或 pnpm / yarn / bun test
```

### 代码检查任务 (Lint Task)
```bash
npm run lint
```

## 依赖解析与指纹计算
- 解析 `package.json` 中的 `dependencies` 与 `devDependencies`。
- 监听 Lock 文件 (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `bun.lockb`)。
- 源码指纹自动排除 `node_modules/` 与 `dist/`。
