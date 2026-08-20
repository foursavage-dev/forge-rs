# Fish API Documentation

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

This document provides API documentation for Fish's main components.

## Table of Contents

- [Core API](#core-api)
- [CLI API](#cli-api)
- [Backend API](#backend-api)
- [Plugin API](#plugin-api)
- [Security API](#security-api)

## Core API

### Workspace Discovery

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

**Methods**:
- `new(name, version, path)`: Create a new package
- `add_dependency(dep)`: Add a dependency
- `is_dependency_of(package)`: Check if this package depends on another

#### Workspace

```rust
pub struct Workspace {
    pub root: PathBuf,
    pub packages: Vec<Package>,
    pub backend: BackendType,
}
```

**Methods**:
- `new(root)`: Create a new workspace
- `discover()`: Discover packages in workspace
- `get_package(name)`: Get a package by name
- `get_build_order()`: Get packages in build order

### Build Graph

#### Graph

```rust
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
```

**Methods**:
- `new()`: Create a new graph
- `add_node(node)`: Add a node to the graph
- `add_edge(from, to)`: Add an edge between nodes
- `topological_sort()`: Get nodes in topological order
- `get_levels()`: Get parallel execution levels

#### Node

```rust
pub struct Node {
    pub id: String,
    pub package: Package,
    pub state: NodeState,
}
```

**Methods**:
- `new(id, package)`: Create a new node
- `with_state(state)`: Set node state

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

**Parameters**:
- `packages`: List of packages to build (empty = all)
- `jobs`: Number of parallel jobs
- `no_cache`: Disable cache
- `sandbox`: Enable sandbox mode

**Returns**: `BuildResult` with build statistics

### Test Command

```rust
pub async fn test(
    packages: Vec<String>,
    no_cache: bool,
) -> Result<TestResult>
```

**Parameters**:
- `packages`: List of packages to test
- `no_cache`: Disable cache

**Returns**: `TestResult` with test statistics

### Graph Command

```rust
pub async fn graph(
    format: GraphFormat,
    output: Option<PathBuf>,
) -> Result<()>
```

**Parameters**:
- `format`: Output format (tree, json, dot)
- `output`: Output file path

**Returns**: Success or error

## Backend API

### Backend Trait

```rust
pub trait Backend {
    fn detect(&self, path: &Path) -> bool;
    fn extract_dependencies(&self, path: &Path) -> Result<Vec<Dependency>>;
    fn generate_tasks(&self, package: &Package) -> Result<Vec<Task>>;
}
```

**Methods**:
- `detect(path)`: Check if backend can handle this path
- `extract_dependencies(path)`: Extract dependencies from project
- `generate_tasks(package)`: Generate build tasks for package

### Dependency

```rust
pub struct Dependency {
    pub name: String,
    pub version: VersionReq,
    pub source: DependencySource,
}
```

**Fields**:
- `name`: Dependency name
- `version`: Version requirement
- `source`: Dependency source (registry, git, path)

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

**Fields**:
- `id`: Task identifier
- `command`: Command specification
- `dependencies`: Task dependencies
- `inputs`: Input files
- `outputs`: Output files

## Plugin API

### Plugin Manager

```rust
pub struct PluginManager {
    plugins: HashMap<String, ScriptPlugin>,
}
```

**Methods**:
- `new()`: Create a new plugin manager
- `load_plugins(path)`: Load plugins from directory
- `execute(plugin, command, args)`: Execute a plugin command
- `list()`: List available plugins

### Script Plugin

```rust
pub struct ScriptPlugin {
    pub name: String,
    pub script_type: ScriptType,
    pub main: PathBuf,
    pub commands: HashMap<String, String>,
}
```

**Fields**:
- `name`: Plugin name
- `script_type`: Script type (Shell, Python, Node, WASM, Lua)
- `main`: Main script file
- `commands`: Available commands

### Script Type

```rust
pub enum ScriptType {
    Shell,
    Python,
    Node,
    WASM,
    Lua,
}
```

## Security API

### Signing Service

```rust
pub struct SigningService {
    keypair: SigningKeyPair,
    algorithm: SignatureAlgorithm,
}
```

**Methods**:
- `new(keypair)`: Create a new signing service
- `sign_artifact(artifact_path, metadata)`: Sign an artifact
- `generate_sbom(package_path, format)`: Generate SBOM
- `public_key()`: Get public key for verification

### Artifact Signature

```rust
pub struct ArtifactSignature {
    pub algorithm: SignatureAlgorithm,
    pub signature: String,
    pub artifact_hash: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: SbomMetadata,
    pub signer_public_key: String,
}
```

**Fields**:
- `algorithm`: Signature algorithm used
- `signature`: Base64-encoded signature
- `artifact_hash`: SHA256 hash of artifact
- `timestamp`: Signing timestamp
- `metadata`: SBOM metadata
- `signer_public_key`: Signer's public key

### Vulnerability Scanner

```rust
pub struct VulnerabilityScanner {
    rust_scanner: RustScanner,
    npm_scanner: NpmScanner,
    maven_scanner: MavenScanner,
}
```

**Methods**:
- `new()`: Create a new scanner
- `scan(project_path, options)`: Scan project for vulnerabilities
- `scan_dependencies(deps, options)`: Scan specific dependencies

### Vulnerability

```rust
pub struct Vulnerability {
    pub id: String,
    pub package: String,
    pub affected_versions: String,
    pub fixed_version: Option<String>,
    pub severity: Severity,
    pub source: VulnerabilitySource,
    pub description: String,
    pub cvss_score: Option<f32>,
}
```

**Fields**:
- `id`: Vulnerability ID (CVE, GHSA, etc.)
- `package`: Affected package name
- `affected_versions`: Version range affected
- `fixed_version`: Version that fixes vulnerability
- `severity`: Severity level
- `source`: Vulnerability source
- `description`: Vulnerability description
- `cvss_score`: CVSS score

## Error Handling

All APIs use `Result<T>` for error handling:

```rust
pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    // Core errors
    PackageNotFound(String),
    InvalidWorkspace(String),
    
    // Build errors
    BuildFailed(String),
    DependencyError(String),
    
    // Security errors
    SigningError(String),
    VerificationError(String),
    
    // IO errors
    IoError(std::io::Error),
}
```

## Examples

### Discover Workspace

```rust
use fish_core::Workspace;

let workspace = Workspace::new(PathBuf::from("/path/to/project"))?;
workspace.discover()?;

for package in workspace.get_build_order() {
    println!("Package: {}", package.name);
}
```

### Build Package

```rust
use fish_cli::build;

let result = build(vec!["my-package".to_string()], 4, false, false).await?;
println!("Build completed in {:.2}s", result.duration);
```

### Sign Artifact

```rust
use fish_signing::SigningService;

let service = SigningService::new(keypair);
let signature = service.sign_artifact(
    PathBuf::from("target/release/my_binary"),
    metadata
).await?;
```

### Scan for Vulnerabilities

```rust
use fish_security::VulnerabilityScanner;

let scanner = VulnerabilityScanner::new();
let report = scanner.scan(PathBuf::from("/path/to/project"), &options).await?;
println!("Found {} vulnerabilities", report.total_vulnerabilities);
```

## Async Support

Most APIs are async to support I/O-bound operations:

```rust
pub async fn build_packages(packages: Vec<String>) -> Result<BuildResult> {
    // Async implementation
}
```

Use with Tokio runtime:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    build_packages(vec!["my-package".to_string()]).await?;
    Ok(())
}
```

## Streaming Support

For large operations, Fish supports streaming:

```rust
pub async fn build_streaming(
    packages: Vec<String>,
) -> Result<impl Stream<Item = BuildEvent>> {
    // Stream build events
}
```

## Extensibility

### Custom Backend

Implement the `Backend` trait:

```rust
struct MyBackend;

impl Backend for MyBackend {
    fn detect(&self, path: &Path) -> bool {
        path.join("my-config.json").exists()
    }
    
    fn extract_dependencies(&self, path: &Path) -> Result<Vec<Dependency>> {
        // Extract dependencies
    }
    
    fn generate_tasks(&self, package: &Package) -> Result<Vec<Task>> {
        // Generate tasks
    }
}
```

### Custom Plugin

Create a plugin script:

```bash
#!/bin/bash
# my-plugin.sh
echo "Building with custom logic"
# Custom build logic
```

Register in `plugin.json`:

```json
{
  "name": "my-plugin",
  "type": "shell",
  "main": "my-plugin.sh",
  "commands": {
    "build": "./my-plugin.sh build"
  }
}
```

## Performance Considerations

- **Async APIs**: Designed for concurrent operations
- **Caching**: Built-in fingerprint-based caching
- **Parallelism**: Automatic parallel execution where possible
- **Memory Efficiency**: Streaming for large operations

## Security Considerations

- **Input Validation**: All paths validated
- **Sandboxing**: Optional sandbox mode
- **Secret Management**: No secrets in logs
- **Verification**: Artifact signature verification

## Versioning

APIs follow semantic versioning. Breaking changes will increment the major version.

## License

MIT License - see [LICENSE](../../LICENSE) for details.
