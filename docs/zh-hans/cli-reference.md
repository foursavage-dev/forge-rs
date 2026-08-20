# CLI 命令行参考

> 🌐 **翻译与贡献：** 想用您的母语翻译或完善本文档？请查看 [翻译指南](TRANSLATION.md)。

Fish 命令行完整命令与参数指南。

## 基础命令

| 命令 | 描述 |
| :--- | :--- |
| `fish init` | 在当前目录下初始化 `fish.toml` 配置文件 |
| `fish new <name>` | 基于预设模板创建新项目或包 |
| `fish build` | 编译构建工作区中所有目标任务 |
| `fish check` | 执行快速语法与类型检查 |
| `fish test` | 并发执行所有单元与集成测试 |
| `fish clean` | 清理构建产物与本地缓存 |

## AI 智能命令

```bash
# 分析构建错误日志
fish ai analyze --toolchain rust --stderr "<log_content>"

# 优化任务调度图
fish ai optimize --workers 8

# 推荐需构建的目标包
fish ai recommend
```

## 网络与分布式命令

```bash
# 启动远程缓存服务端
fish cache-server --listen 0.0.0.0:8080

# 启动分布式 Worker 节点
fish worker --coordinator http://coordinator:9090
```

## 诊断与辅助工具

```bash
# 运行环境依赖全面检查
fish doctor --ai

# 查询包依赖关系
fish query "deps(//packages/core)"

# 文件变动实时监听构建
fish watch
```
