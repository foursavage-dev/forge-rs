# Fish 系统架构指南

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](TRANSLATION.md)。

本文档全面介绍了 Fish 的系统架构、核心引擎模块以及执行流水线。

---

## 系统概览

Fish 是一个专为多语言 Monorepo 及分布式开发环境设计的高性能、缓存优先构建编排系统。Fish 并不取代各语言的原生编译器，而是作为统一的智能协调层，负责管理依赖 DAG 图、内容寻址存储（CAS）、沙箱隔离以及并发工作窃取（work-stealing）调度。

```text
┌─────────────────────────────────────────────────────────────┐
│                    fish-cli / Web UI                        │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│       fish-core (Discovery, Toolchains, compile_commands)   │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│           fish-graph (DAG & Algebraic Query Engine)         │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│   fish-scheduler (Governor, Jobserver, Racing, Watcher)     │
└──────────────┬──────────────────────────────┬───────────────┘
               │                              │
┌──────────────▼──────────────┐┌──────────────▼──────────────┐
│ fish-executor & Middleware  ││  fish-cache & fish-cas      │
└──────────────┬──────────────┘└──────────────┬──────────────┘
               │                              │
┌──────────────▼──────────────────────────────▼──────────────┐
│      11+ Language Backends & Distributed Worker Network     │
└─────────────────────────────────────────────────────────────┘
```

---

## 核心组件与职责划分

### 1. 工作区发现 (`fish-core`)
- **清单解析与发现**：扫描并解析 `Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, `CMakeLists.txt`, `pom.xml`, `*.csproj`, `Package.swift`, `pubspec.yaml`, `build.zig`, `Dockerfile`。
- **编译数据库生成**：为 Clangd 及主流 IDE 生成标准的 `compile_commands.json` 文件 (`CompilationDatabase`)。
- **密封工具链管理**：管理并隔离各编译工具链的二进制路径与环境变量 (`ToolchainRegistry`, `ToolchainSpec`)。
- **微输入过滤 (Micro-Input Filtering)**：基于 Glob 模式精确过滤输入文件，避免无关文件变动触发缓存失效 (`MicroInputFilter`)。

### 2. 构建依赖图 (`fish-graph`)
- **拓扑任务图**：构建任务的有向无环图 (DAG) 并进行循环依赖检测。
- **代数图查询引擎**：执行 Bazel 风格的图表达式 (`deps()`, `rdeps()`, `allpaths()`, `somepath()`, `filter()`)。
- **动态节点展开**：在执行期间动态生成并插入子任务图 (`DynamicGraphExpander`)。

### 3. 执行与高速实体化 (`fish-executor`)
- **进程调度与监控**：支持超时控制与输出流捕获的非阻塞异步任务执行。
- **快速数据克隆 (Fast Extents Cloning)**：利用写时复制 (CoW) 与硬链接极速实体化产物，零磁盘 I/O 复制开销 (`KernelCowCloner`)。
- **链接器智能调度**：自动检测并适配 `mold`, `lld`, `lld-link` 与 `msvc` 的链接参数 (`LinkerDispatcher`)。
- **编译器响应参数文件**：在命令行参数超出操作系统限制时自动生成 `@fish_args.rsp` 响应参数文件。

### 4. 调度器与系统资源管控 (`fish-scheduler`)
- **并行工作窃取 (Parallel Work-Stealing)**：在所有可用硬件核心上实现无锁高效调度。
- **内核资源管控器 (Kernel Resource Governor)**：实时监控物理内存占用，动态限制并发数，杜绝内存溢出 (OOM) 崩溃 (`KernelResourceGovernor`)。
- **流水线式级联编译 (Compiler Pipelining)**：在元数据就绪后立即解锁下游任务，缩短总体关键路径耗时 (`PipelinedCompilationCoordinator`)。
- **GNU Jobserver 令牌池**：跨多进程与嵌套编译器的全局线程令牌管理池 (`JobserverPool`)。
- **动态远程竞速 (Dynamic Remote Racing)**：同时在本地与远程集群节点执行，取最快完成者 (`DynamicRacingExecutor`)。
- **分布式任务装箱 (DTE)**：采用最长处理时间 (LPT) 装箱算法实现多 CI Worker 负载均衡 (`DteBinPacker`)。
- **实时文件系统监听器**：后台守护进程监听文件变动并提前预热缓存依赖图 (`FsWatcherDaemon`)。

### 5. 内容寻址缓存 (`fish-cache` & `fish-cas`)
- **高保真指纹计算 (Fingerprinting)**：基于 Blake3 对源码、环境变量与编译器标志进行哈希计算。
- **CAS 去重存储**：基于 Zstandard 高速压缩的全局产物去重存储。
- **分层复合缓存架构**：支持 L1 本地内存/磁盘缓存与 L2 远程 S3/HTTP 缓存的无缝集成。

### 6. 用户界面与遥测 (`fish-cli`)
- **命令行界面**：提供针对 build, test, check, graph, doctor, query, affected 及 daemon 管理的完整子命令。
- **交互式 SVG DAG 可视化仪表板**：支持平移、缩放、节点聚焦、关键路径高亮等交互操作的实时 Web 画布。
- **内置 5 语言国际化**：原生内置英语、越南语、简体中文、繁体中文和日语。
- **本地守护进程 IPC**：基于 `127.0.0.1:9527` 的回环 TCP 守护进程，实现亚毫秒级热图解析。

---

## 支持的语言后端

Fish 内置了 11 种专用语言适配器：

| 后端名称 | 标识符 | 主要清单文件 | 默认编译器 / 工具 |
| :--- | :--- | :--- | :--- |
| **Rust** | `rust` | `Cargo.toml` | `cargo`, `rustc` |
| **C / C++** | `cc` | `CMakeLists.txt`, `Makefile` | `cmake`, `clang`, `gcc`, `msvc` |
| **Go** | `go` | `go.mod` | `go build`, `go test` |
| **TypeScript / Node** | `ts` | `package.json` | `npm`, `pnpm`, `yarn`, `bun` |
| **Python** | `py` | `pyproject.toml`, `requirements.txt` | `python -m build`, `pytest`, `uv` |
| **Java / Kotlin** | `java` | `pom.xml`, `build.gradle` | `mvn`, `gradle` |
| **.NET** | `dotnet` | `*.csproj`, `*.sln` | `dotnet build`, `dotnet test` |
| **Swift** | `swift` | `Package.swift` | `swift build`, `swift test` |
| **Dart / Flutter** | `dart` | `pubspec.yaml` | `dart compile`, `flutter build` |
| **Zig** | `zig` | `build.zig` | `zig build` |
| **Docker** | `docker` | `Dockerfile` | `docker build` |

---

## 安全与产物校验

- **产物密码学签名 (`fish-signing`)**：采用 Ed25519 算法生成与验证数字签名。
- **SBOM 软件物料清单导出**：支持 SPDX 与 CycloneDX 标准格式。
- **依赖漏洞扫描 (`fish-security`)**：基于 CVSS 评分自动扫描依赖漏洞并可按严重程度阻断构建。
- **机密管理与脱敏 (`fish-secrets`)**：与 HashiCorp Vault、AWS Secrets Manager、Kubernetes Secret 集成，并自动在终端日志中脱敏敏感信息。
