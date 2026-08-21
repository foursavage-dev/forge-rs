# Fish CLI 命令行参考大全

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](TRANSLATION.md)。

Fish 命令行界面的完整命令与选项参考指南。

---

## 全局选项 (Global Options)

- `--experimental`: 启用实验性功能。
- `-v, --verbose`: 启用详细诊断日志输出。
- `-j, --jobs <N>`: 最大并发工作线程数。
- `--no-cache`: 绕过本地与远程缓存。
- `--cache-dir <PATH>`: 自定义本地缓存目录。
- `--explain`: 打印目标重新构建的详细原因。
- `--pgo-generate`: 为 Profile-Guided Optimization (PGO) 生成插桩二进制文件。
- `--pgo-use`: 使用采集的 PGO 性能剖析数据进行优化编译。

---

## 核心命令

### `fish init`
初始化 Fish 配置并扫描工作区以生成多语言任务定义 (`fish.yaml`)。

```bash
fish init [--force]
```

---

### `fish ui`
启动内置的实时 Web 仪表板与 SVG DAG 依赖图可视化工具，支持 5 种语言（英语、越南语、简体中文、繁体中文、日语）。

```bash
fish ui [--port <PORT>] [--open]
```

---

### `fish build`
执行工作区内软件包的构建任务。

```bash
fish build [OPTIONS]
```

**常用标志：**
- `-p, --package <NAME>`: 构建指定的软件包。
- `--explain`: 诊断软件包为何需要重新编译。
- `--profile [FILE]`: 生成 Chrome Trace JSON 格式的性能分析数据。
- `--sandbox`: 在隔离沙箱环境中执行构建。
- `--ram-limit <PCT>`: 当物理内存占用达到阈值时动态限制并发度。

---

### `fish check`
执行类型检查和静态分析，而不进行完整的产物链接。

```bash
fish check [OPTIONS]
```

---

### `fish test`
运行工作区内各软件包的测试套件。

```bash
fish test [OPTIONS]
```

---

### `fish run`
编译并运行指定的二进制目标。

```bash
fish run -p <PACKAGE> --bin <BINARY> [-- <ARGS>...]
```

---

### `fish query <EXPR>`
对工作区依赖图执行代数表达式查询。

```bash
fish query "<EXPRESSION>"
```

**支持的函数：**
- `deps(//pkg)`: `//pkg` 的所有传递依赖。
- `rdeps(//pkg)`: 依赖 `//pkg` 的所有反向依赖。
- `allpaths(//from, //to)`: `//from` 到 `//to` 之间的所有路径。
- `somepath(//from, //to)`: `//from` 到 `//to` 之间的最短路径。
- `filter('pattern', expr)`: 按关键字或正则模式过滤匹配项。

**示例：**
```bash
# 查询构建 fish-cli 所需的所有依赖项
fish query "deps(//fish-cli)"

# 查询受 fish-graph 改动影响的所有下游模块
fish query "rdeps(//fish-graph)"

# 查询 app 到 util 之间的最短依赖链路
fish query "somepath(//app, //util)"
```

---

### `fish daemon`
管理后台构建守护进程，实现毫秒级热图解析。

```bash
# 启动守护进程
fish daemon start [--port 9527]

# 查看守护进程状态
fish daemon status [--port 9527]

# 停止守护进程
fish daemon stop [--port 9527]
```

---

### `fish graph`
输出或导出项目的依赖图结构。

```bash
fish graph [--format <tree|dot|json>]
```

---

### `fish affected`
识别自某个 Git 提交节点以来发生修改的软件包，并仅对其执行任务。

```bash
fish affected --since <GIT_REF> [--mode <build|check|test>]
```

---

### `fish cache`
管理本地内容寻址存储（CAS）与构建指纹。

```bash
# 显示缓存占用空间与对象数量
fish cache stats

# 清理陈旧指纹与孤立产物
fish cache prune

# 检查 CAS 存储状态
fish cache cas stats
fish cache cas list
```

---

### `fish doctor`
检测系统工具链、编译器、链接器与依赖环境的就绪状态。

```bash
fish doctor [--fix] [--ai]
```

---

### `fish ci init` / `fish ci export`
为多种 CI/CD 平台自动生成流水线配置。

```bash
fish ci init --platform <github|gitlab|circleci|bitbucket|all>
```
