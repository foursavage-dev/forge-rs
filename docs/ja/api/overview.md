# Fish API リファレンス

> 🌐 **翻訳と貢献:** このドキュメントをあなたの言語で翻訳または改善したいですか？ [翻訳ガイドライン](../TRANSLATION.md) をご覧ください。

このドキュメントでは、Fish の主要コンポーネントに関するプログラミングインターフェイス (API) の詳細を解説します。

## 目次

- [Core API](#core-api)
- [CLI API](#cli-api)
- [Backend API](#backend-api)
- [Plugin API](#plugin-api)
- [Security API](#security-api)

## Core API

### ワークスペース検出 (Workspace Discovery)

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

**メソッド**:
- `new(name, version, path)`: パッケージを作成。
- `add_dependency(dep)`: 依存関係を追加。
- `is_dependency_of(package)`: 他パッケージへの依存関係を判定。

#### Workspace

```rust
pub struct Workspace {
    pub root: PathBuf,
    pub packages: Vec<Package>,
    pub backend: BackendType,
}
```

**メソッド**:
- `new(root)`: ワークスペースを作成。
- `discover()`: ワークスペース内のパッケージを検出。
- `get_package(name)`: パッケージ名で取得。
- `get_build_order()`: トポロジカル順序でビルド順を取得。

### ビルドグラフ (Build Graph)

#### Graph & Node

```rust
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub struct Node {
    pub id: String,
    pub package: Package,
    pub state: NodeState,
}
```

## CLI API

```rust
pub async fn build(
    packages: Vec<String>,
    jobs: usize,
    no_cache: bool,
    sandbox: bool,
) -> Result<BuildResult>
```

## Backend API

```rust
pub trait Backend {
    fn detect(&self, path: &Path) -> bool;
    fn extract_dependencies(&self, path: &Path) -> Result<Vec<Dependency>>;
    fn generate_tasks(&self, package: &Package) -> Result<Vec<Task>>;
}
```
