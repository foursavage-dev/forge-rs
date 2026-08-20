# Fish 快速入门

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](TRANSLATION.md)。

本指南将帮助您快速上手 Fish —— 一个快速、缓存优先的构建编排系统。

## 安装

### 一键安装（推荐）

**Linux & macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/requla11/fish/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/requla11/fish/main/install.ps1 | iex
```

### 源码编译安装

```bash
# 克隆仓库
git clone https://github.com/requla11/fish.git
cd fish

# 编译并安装
cargo install --path crates/fish-cli
```

### 通过 Cargo 安装

```bash
cargo install fish-cli --git https://github.com/requla11/fish
```

## 快速上手

### 构建 Rust 项目

```bash
cd your-rust-project
fish build
```

### 构建多语言 Monorepo

```bash
# 克隆示例仓库
git clone https://github.com/requla11/fish.git
cd fish/examples/polyglot-demo

# 构建所有服务
fish build

# 查看构建依赖图
fish graph

# 运行测试
fish test
```

## 常用命令

### 构建命令

```bash
# 构建整个工作区
fish build

# 构建指定软件包
fish build -p my-package

# 使用 8 个并发任务构建
fish build -j 8

# 禁用缓存构建
fish build --no-cache

# 在沙箱隔离环境中构建
fish build --sandbox

# 输出详细的重新构建原因
fish build --explain

# 基于性能剖析的优化 (PGO) 流程
fish build --pgo-generate
# ... 运行您的基准测试或负载 ...
fish build --pgo-use
```

### 图与查询命令 (Graph & Query)

```bash
# 查询传递依赖（Bazel 风格）
fish query "deps(//fish-cli)"

# 查询反向依赖
fish query "rdeps(//fish-graph)"

# 查找两个模块之间的所有路径
fish query "allpaths(//fish-cli, //fish-core)"

# 使用正则表达式过滤依赖
fish query "filter('backend', deps(//fish-cli))"

# 可视化依赖图渲染
fish graph --format tree
fish graph --format dot
```

### 构建守护进程命令 (Daemon)

```bash
# 启动后台守护进程，实现毫秒级热构建
fish daemon start

# 查看守护进程状态
fish daemon status

# 停止后台守护进程
fish daemon stop
```

### 测试命令

```bash
# 运行所有测试
fish test

# 测试指定软件包
fish test -p my-package

# 禁用缓存运行测试
fish test --no-cache
```

### 缓存管理命令

```bash
# 查看缓存统计信息
fish cache stats

# 清理过期缓存
fish cache prune

# 启动远程缓存服务器
fish cache-server --listen 0.0.0.0:8080
```

### 分布式构建命令

```bash
# 启动工作节点 (Worker)
fish worker --listen 0.0.0.0:9000

# 使用分布式集群构建
fish build --workers worker1:9000,worker2:9000
```

### CI/CD 配置生成命令

```bash
# 生成 GitHub Actions 工作流
fish ci init --platform github

# 生成 GitLab CI 流水线
fish ci init --platform gitlab

# 生成 CircleCI 配置
fish ci init --platform circleci

# 生成 Bitbucket Pipelines
fish ci init --platform bitbucket

# 生成所有支持平台的配置
fish ci init --platform all
```

### 插件命令

```bash
# 列出所有可用插件
fish plugin list

# 执行插件命令
fish plugin execute my-plugin build

# 安装插件
fish plugin install ./my-plugin
```

## 项目配置

### 工作区配置 (`fish.toml`)

Fish 会根据项目清单文件自动识别项目类型。如需自定义执行参数、缓存及流水线，可在项目根目录创建 `fish.toml`：

```toml
[build]
backend = "auto"
jobs = 8
no_cache = false
sandbox = false
semantic = true
critical_path = true
ram_limit = 85

[cache]
dir = "~/.fish/cache"
reflink = true

[remote]
cache_url = "http://127.0.0.1:8080"
token = "secret-cache-token"

[daemon]
port = 9527

[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "Cargo.toml"]
outputs = ["target/release/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]
```

完整配置项请参阅 [配置指南](configuration.md)。

---

## 交互式遥测与 Web 仪表板

Fish 内置了实时交互式 DAG 可视化分析工具与 Web 仪表板，支持 5 种语言（英语、越南语、简体中文、繁体中文、日语）：

```bash
# 在 3000 端口启动 Web 仪表板并自动在浏览器中打开
fish ui --port 3000 --open

# 查看 JSON 格式的依赖图数据
curl http://localhost:3000/api/graph

# 查看硬件利用率与 CAS 统计信息
curl http://localhost:3000/api/stats
```

---

## 常见问题与排错

### 构建失败

如果构建失败：

1. 检查错误信息或运行 `fish build --explain` 诊断重新编译原因。
2. 开启调试日志运行：`RUST_LOG=debug fish build`
3. 检查工具链就绪状态：`fish doctor`
4. 尝试清理缓存：`fish cache prune`

### 缓存失效或异常

如果缓存未能命中或工作异常：

1. 查看缓存状态：`fish cache stats`
2. 确认缓存目录可写：`~/.fish/cache`
3. 清理并重新构建：`fish cache prune && fish build`

### Worker 节点连接失败

如果分布式 Worker 无法连接：

1. 检查节点之间的网络连通性
2. 确认 Worker 正在运行：`fish worker --listen 0.0.0.0:9000`
3. 检查防火墙设置与认证 Token
4. 查看 Worker 端的日志输出

## 下一步

- 阅读 [系统架构指南](architecture.md)
- 查阅 [开发指南](../DEVELOPMENT.md)
- 探索 [CLI 命令参考](cli-reference.md)
- 查看 [各语言后端文档](backends/)

## 获取帮助

- [官方文档](../README.md)
- [技术支持](../SUPPORT.md)
- [GitHub Issues 反馈](https://github.com/requla11/fish/issues)
- [联系邮箱](foursavage@proton.me)
