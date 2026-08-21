# 36 Crates 核心工作区架构 (`crates/`)

Fish 由 36 个高度模块化的 Rust Crates 组成，分层严谨且高内聚低耦合。

## 架构分层
1. **基础层 (Foundation Tier)**:
   - `fish-core`: 项目探测、Manifest 解析、`fish.toml` 配置。
   - `fish-graph`: DAG 图模型、无锁拓扑排序、图查询代数。
   - `fish-executor`: OS 子进程执行、`@args.rsp` 响应参数文件、中间件链。
2. **存储与缓存层 (Storage & Cache Tier)**:
   - `fish-cas`: ZSTD 压缩与 FastCDC 分块的 CAS 存储。
   - `fish-cache`: 双阶段 Fingerprint 缓存与 GC 清理。
   - `fish-remote-cache`: gRPC REAPI v2 客户端与 TCP 流式缓存。
3. **调度与执行层 (Scheduling Tier)**:
   - `fish-scheduler`: 关键路径动态预测调度器、Chase-Lev 工作窃取队列、GNU Jobserver。
   - `fish-worker`: 远程 Worker 集群执行与 Daemon IPC。
   - `fish-sandbox`: Linux eBPF 系统调用追踪与 WASM 沙箱。
4. **11 种语言后端适配器**:
   - Rust, C++, Go, TS, Python, Docker, Java, .NET, Swift, Dart, Zig。
5. **安全与诊断工具层**:
   - `fish-security`, `fish-signing`, `fish-secrets`, `fish-flaky-detection`, `fish-notifications`, `fish-analytics`, `fish-templates`, `fish-docker-builder`, `fish-incremental`, `fish-multiplatform`, `fish-installer`。
6. **顶层 CLI 应用**:
   - `fish-cli`: 统一 CLI 入口、TUI 瀑布流仪表盘与 `fish lsp` 语言服务端。
