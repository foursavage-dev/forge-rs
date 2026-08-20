# Fish API 介面文件

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](../TRANSLATION.md)。

本文档提供了 Fish 核心组件的详细编程接口 (API) 说明。

## 目录

- [Core API 核心接口](#core-api)
- [CLI API 命令行接口](#cli-api)
- [Backend API 后端接口](#backend-api)
- [Plugin API 插件接口](#plugin-api)
- [Security API 安全接口](#security-api)

## Core API

### 工作區发现 (Workspace Discovery)

#### Package (套件)

```rust
pub struct Package {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub dependencies: Vec<Dependency>,
    pub backend: BackendType,
}
```

**核心方法**:
- `new(name, version, path)`: 创建新套件。
- `add_dependency(dep)`: 添加依赖项。
- `is_dependency_of(package)`: 检查此套件是否依赖另一套件。

#### Workspace (工作區)

```rust
pub struct Workspace {
    pub root: PathBuf,
    pub packages: Vec<Package>,
    pub backend: BackendType,
}
```

**核心方法**:
- `new(root)`: 创建新工作區实例。
- `discover()`: 扫描并发现工作區中的所有套件。
- `get_package(name)`: 根据名称获取套件。
- `get_build_order()`: 获取拓扑排序后的构建顺序。

### 构建依赖图 (Build Graph)

#### Graph

```rust
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
```

**核心方法**:
- `new()`: 创建新依赖图。
- `add_node(node)`: 添加节点。
- `add_edge(from, to)`: 在节点之间添加依赖边。
- `topological_sort()`: 获取拓扑排序节点列表。
- `get_levels()`: 获取并发执行分层。

#### Node & NodeState

```rust
pub struct Node {
    pub id: String,
    pub package: Package,
    pub state: NodeState,
}

pub enum NodeState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
}
```

## CLI API

### Build 构建命令

```rust
pub async fn build(
    packages: Vec<String>,
    jobs: usize,
    no_cache: bool,
    sandbox: bool,
) -> Result<BuildResult>
```

### Test 测试命令

```rust
pub async fn test(
    packages: Vec<String>,
    no_cache: bool,
) -> Result<TestResult>
```

## Backend API

### Backend Trait 接口

```rust
pub trait Backend {
    fn detect(&self, path: &Path) -> bool;
    fn extract_dependencies(&self, path: &Path) -> Result<Vec<Dependency>>;
    fn generate_tasks(&self, package: &Package) -> Result<Vec<Task>>;
}
```

### Task 任务结构

```rust
pub struct Task {
    pub id: String,
    pub command: CommandSpec,
    pub dependencies: Vec<String>,
    pub inputs: Vec<PathBuf>,
    pub outputs: Vec<PathBuf>,
}
```

## Plugin API & Security API

### PluginManager

```rust
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}
```

### VulnerabilityScanner

```rust
pub struct VulnerabilityScanner {
    advisory_database: AdvisoryDb,
}
```
