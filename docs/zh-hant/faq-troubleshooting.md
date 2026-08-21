# 常见问题与故障排除

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](TRANSLATION.md)。

本文档汇总了 Fish 的常见问题解答、迁移建议以及故障排除方案。

---

## 常见问题解答 (FAQ)

### 1. Fish 会取代 Cargo, npm 或 go build 吗？
不会。Fish 是一个构建**编排系统 (Orchestrator)**，而不是编译器的替代品。它协调您现有的工具链（Cargo, rustc, Node.js, Go, GCC/Clang, dotnet），分析统一的依赖图，并通过密封缓存、并行调度和分布式执行来加速构建。

### 2. 如何将现有的 Monorepo 迁移到 Fish？
Fish 会自动通过清单文件识别项目 (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, `CMakeLists.txt`, `pom.xml`, `*.csproj`)。
1. 进入您的项目根目录。
2. 运行 `fish build` 让 Fish 自动发现整个工作區。
3. （可选）在根目录下创建 `fish.toml` 以自定义流水线依赖关系和缓存路径。

### 3. Fish 的 CAS 内容寻址缓存是如何工作的？
Fish 会对输入文件、工具链版本和環境變量计算 Blake3 唯一指纹。当任务生成输出产物时，产物会经由 Zstandard 压缩并存入内容寻址存储目录 (`~/.fish/cache`)。如果输入未发生变化，Fish 将直接通过写时复制 (CoW) 或硬链接极速还原产物，完全跳过编译器的调用。

---

## 故障排除方案

### 问题：目标未按预期命中缓存，频繁重新编译
**解决方案：**
使用 `--explain` 标志查看目标被判定为脏状态的具体原因：
```bash
fish build --explain
```
常见原因包括：
- 某个源文件的时间戳或内容发生了变动。
- 上游依赖模块的输出哈希发生了改变。
- 環境變量的差异导致缓存指纹失效。

---

### 问题：多任务并发构建时物理内存占用过高
**解决方案：**
当同时编译多个大型 Crate 或 C++ 模块时，内存压力可能导致磁盘 Swap 颠簸。可以使用 `--ram-limit` 标志或在 `fish.toml` 中配置 `ram_limit`：
```bash
fish build --ram-limit 80
```
Fish 的资源管控器将在内存占用超过阈值时自动節流，减少并发任务数。

---

### 问题：后台守護行程端口冲突 (`9527`)
**解决方案：**
若端口 `9527` 已被其他进程占用，请指定自定义端口：
```bash
fish daemon start --port 9588
```
或者设置環境變量：
```bash
export FISH_DAEMON_PORT=9588
```

---

### 问题：Windows 系统下文件锁冲突 (`os error 5: Access is denied`)
**解决方案：**
在 Windows 系统中，直接运行 `target/debug` 目录下的可执行文件会导致该二进制文件被系统锁定。请将 Fish 全局安装至 `%USERPROFILE%\.cargo\bin`：
```bash
cargo install --path crates/fish-cli --force
```
然后直接在任意目录下调用 `fish` 命令。
