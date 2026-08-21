# 对比矩阵：Fish 与其他主流構建系統

Fish 使用 Rust 2024 从底层精心打造，专为现代多語言 Monorepo 设计。以下是 Fish 与 Bazel、Turborepo 以及 Buck2 的全面横向对比：

| 功能 / 维度 | Fish | Turborepo | Bazel | Buck2 |
| :--- | :--- | :--- | :--- | :--- |
| **核心实现语言** | Rust 2024 | Go / Rust | Java / C++ | Rust |
| **多語言支持** | 原生多語言（11+ 工具链） | 主打 JS / TS | 多語言 (Starlark 规则) | 多語言 (Starlark 规则) |
| **設定模型** | 统一 `fish.toml` / 自动发现 | `turbo.json` | `WORKSPACE` + `BUILD.bazel` | `BUCK` 文件 |
| **上手与設定难度** | 极低（零設定自动识别） | 较低 | 极高 | 较高 |
| **哈希引擎** | Blake3（极速多线程哈希） | SHA-256 | SHA-256 | Blake3 / SHA-256 |
| **CAS 压缩与缓存** | Zstandard (Zstd) + CoW | Tar.gz / Gzip | Zstd / 自定义 | Zstd / 自定义 |
| **产物落地机制** | Reflink / 写时复制 (0ms) | 普通文件复制 | 软链接 / 硬链接 | Reflink / CoW |
| **内容分块引擎** | FastCDC（16KB - 256KB 块去重） | 完整归档文件 | 完整归档文件 | 分块 CAS |
| **VFS 脏文件解析** | 内存快照树（<2ms 级解析） | 磁盘遍历 | Inotify / Watchman 守护进程 | Watchman / EdenFS |
| **语义化缓存失效** | AST 接口签名哈希 (ABI) | 仅文件哈希 | 仅头文件编译 | Header / rmeta 编译 |
| **AI 智能诊断** | 原生 IPC + 故障根因分析 | 无 | 无 | 无 |
| **可视化监控儀表板** | 原生内置 Web GUI 与 TUI | Vercel 网页儀表板 | 第三方工具集成 | 开源命令行终端 |
