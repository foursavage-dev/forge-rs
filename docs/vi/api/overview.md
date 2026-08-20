# Tài liệu API Fish

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](../TRANSLATION.md).

Tài liệu này cung cấp chi tiết giao diện lập trình (API) cho các thành phần chính trong hệ thống Fish.

## Mục lục

- [Core API](#core-api)
- [CLI API](#cli-api)
- [Backend API](#backend-api)
- [Plugin API](#plugin-api)
- [Security API](#security-api)

## Core API

### Khám phá Không gian làm việc (Workspace Discovery)

#### Package

```rust
pub struct Package {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub dependencies: Vec<Dependency>,
    pub backend: BackendType,
}
```

**Phương thức**:
- `new(name, version, path)`: Khởi tạo package mới.
- `add_dependency(dep)`: Thêm một phụ thuộc.
- `is_dependency_of(package)`: Kiểm tra xem package này có phụ thuộc vào package khác hay không.

#### Workspace

```rust
pub struct Workspace {
    pub root: PathBuf,
    pub packages: Vec<Package>,
    pub backend: BackendType,
}
```

**Phương thức**:
- `new(root)`: Khởi tạo workspace mới.
- `discover()`: Quét và khám phá các package trong workspace.
- `get_package(name)`: Lấy thông tin package theo tên.
- `get_build_order()`: Lấy danh sách package theo thứ tự biên dịch topo.

### Đồ thị Biên dịch (Build Graph)

#### Graph

```rust
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
```

**Phương thức**:
- `new()`: Khởi tạo đồ thị mới.
- `add_node(node)`: Thêm một node vào đồ thị.
- `add_edge(from, to)`: Thêm cạnh phụ thuộc giữa 2 node.
- `topological_sort()`: Lấy các node theo thứ tự topo.
- `get_levels()`: Lấy các tầng thực thi song song.

#### Node

```rust
pub struct Node {
    pub id: String,
    pub package: Package,
    pub state: NodeState,
}
```

#### NodeState

```rust
pub enum NodeState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
}
```

## CLI API

### Build Command

```rust
pub async fn build(
    packages: Vec<String>,
    jobs: usize,
    no_cache: bool,
    sandbox: bool,
) -> Result<BuildResult>
```

**Tham số**:
- `packages`: Danh sách package cần build (rỗng = toàn bộ).
- `jobs`: Số lượng luồng worker song song.
- `no_cache`: Tắt việc tìm kiếm trên cache.
- `sandbox`: Bật chế độ cách ly sandbox.

**Trả về**: `BuildResult` chứa số liệu thống kê build.

### Test Command

```rust
pub async fn test(
    packages: Vec<String>,
    no_cache: bool,
) -> Result<TestResult>
```

## Backend API

### Backend Trait

```rust
pub trait Backend {
    fn detect(&self, path: &Path) -> bool;
    fn extract_dependencies(&self, path: &Path) -> Result<Vec<Dependency>>;
    fn generate_tasks(&self, package: &Package) -> Result<Vec<Task>>;
}
```

**Phương thức**:
- `detect(path)`: Kiểm tra xem backend có xử lý được đường dẫn này không.
- `extract_dependencies(path)`: Trích xuất các phụ thuộc từ dự án.
- `generate_tasks(package)`: Sinh danh sách tác vụ build cho package.

### Dependency

```rust
pub struct Dependency {
    pub name: String,
    pub version: VersionReq,
    pub source: DependencySource,
}
```

### Task

```rust
pub struct Task {
    pub id: String,
    pub command: CommandSpec,
    pub dependencies: Vec<String>,
    pub inputs: Vec<PathBuf>,
    pub outputs: Vec<PathBuf>,
}
```

## Plugin API

### Plugin Manager

```rust
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}
```

**Phương thức**:
- `register(plugin)`: Đăng ký một plugin mới.
- `execute_hook(hook, context)`: Thực thi hook của plugin.

## Security API

### Vulnerability Scanner

```rust
pub struct VulnerabilityScanner {
    advisory_database: AdvisoryDb,
}
```

**Phương thức**:
- `scan_dependencies(deps)`: Quét lỗ hổng trên danh sách phụ thuộc.
- `generate_sbom(format)`: Xuất danh mục thành phần phần mềm (SPDX/CycloneDX).
