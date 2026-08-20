# Fish 系统架构设计

> 🌐 **翻译与贡献：** 想用您的母语翻译或完善本文档？请查看 [翻译指南](TRANSLATION.md)。

Fish 采用专为现代化大型代码仓库（Monorepo）设计的 **三引擎架构 (Rust + Python + Go)**，兼具极致构建速度、云原生分布式扩展以及 AI 智能化分析。

## 三引擎架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI 命令行                          │
│                      (crates/fish-cli)                      │
└──────────────┬──────────────────────────────┬───────────────┘
               │                              │
               ▼                              ▼
┌──────────────────────────────┐ ┌────────────────────────────┐
│      Rust 执行核心 (75%)     │ │      Go 网络调度服务 (10%)  │
│  - fish-core, fish-graph     │ │  - fish-coordinator       │
│  - fish-executor, scheduler  │ │  - fish-worker-gateway    │
│  - fish-cache, fish-cas      │ │  - fish-network, migrator │
└──────────────┬───────────────┘ └────────────┬───────────────┘
               │                              │
               ▼                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Python AI 智能服务 (15%)                │
│   - fish_ai_analyzer   - fish_optimizer                     │
│   - fish_analytics     - fish_recommender                   │
└─────────────────────────────────────────────────────────────┘
```

### 1. Rust 核心执行引擎 (75%)
- **`fish-core`**: 工作区自动发现、配置清单解析以及微文件过滤。
- **`fish-graph`**: 有向无环图（DAG）、拓扑排序与代数依赖查询引擎。
- **`fish-executor`**: 进程控制、沙箱隔离以及中间件流水线。
- **`fish-scheduler`**: 基于 GNU Jobserver 的高并发工作窃取调度器。
- **`fish-cache` & `fish-cas`**: Blake3 多层指纹缓存与 ZSTD 压缩存储。

### 2. Python AI 智能层 (15%)
- **`fish_ai_analyzer`**: 构建失败日志分类、根因定位与修复建议。
- **`fish_optimizer`**: 关键路径（Critical Path）计算与内存约束调度。
- **`fish_analytics`**: 构建耗时遥测聚合与瓶颈检测。
- **`fish_recommender`**: 变更影响分析与不稳定测试（Flaky Tests）检测。

### 3. Go 云原生网络层 (10%)
- **`fish-coordinator`**: 节点注册中心、心跳监控与分布式任务分发。
- **`fish-worker-gateway`**: 高性能反向代理与 Least-Loaded 负载均衡。
- **`fish-network`**: 连接池管理与 mTLS 传输安全。
- **`fish-db-migrator`**: 遥测数据库版本迁移工具。
