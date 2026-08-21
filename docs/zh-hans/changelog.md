# 更新日志与版本历史

此处记录 Fish 项目的所有重要变更。

## [v0.3.0] - 2026-08-21
### 新增特性
- **IDE 扩展**: 官方 VS Code 扩展与 JetBrains 全家桶插件。
- **LSP 协议支持**: `fish lsp` 语言服务端与实时诊断。
- **gRPC REAPI v2**: 分布式 Remote Execution API v2 真实动作执行。
- **eBPF 追踪**: Linux 内核级动态依赖发现。
- **Doctor AI**: 智能环境自愈诊断 (`fish doctor --fix`)。
- **TUI 瀑布流仪表盘**: 实时 CPU/RAM 与任务执行可视化。

## [v0.2.0] - 2026-08-10
### 新增特性
- **Tri-Engine 架构**: Rust 2024 核心 + Go 协调器 + Python AI 引擎。
- **11 种语言后端**: Rust, Go, TS, Python, C++, Docker, Java, .NET, Swift, Dart, Zig。
- **BLAKE3 CAS 存储**: 高性能 ZSTD 内容寻址缓存。
