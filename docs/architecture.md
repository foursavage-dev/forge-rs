# Forge Architecture

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../TRANSLATION.md).

This document provides an in-depth technical overview of Forge's architecture.

## System Overview

Forge is a distributed, cache-first build orchestration system designed for polyglot monorepos. It uses a dependency graph, parallel scheduler, executor, and CAS artifact cache to optimize build performance.

## Core Components

### 1. Workspace Discovery (`forge-core`)

**Purpose**: Automatically discover and model project structure

**Architecture**:
```
WorkspaceDiscovery
    ├── Scanner (traverses filesystem)
    ├── ManifestParser (reads config files)
    ├── DependencyResolver (builds dependency graph)
    └── PackageBuilder (creates Package models)
```

**Key Components**:
- `Workspace`: Root container for all packages
- `Package`: Individual project/unit with metadata
- `Manifest`: Project configuration (Cargo.toml, package.json, etc.)
- `Dependency`: Relationship between packages

### 2. Build Graph (`forge-graph`)

**Purpose**: Model build dependencies and execution order

**Architecture**:
```
BuildGraph
    ├── Graph (DAG structure)
    ├── TopologicalSorter (execution order)
    ├── LevelCalculator (parallel execution groups)
    └── CycleDetector (circular dependency detection)
```

**Data Flow**:
1. Packages → Nodes in graph
2. Dependencies → Edges in graph
3. Topological sort → Execution order
4. Level calculation → Parallel groups

### 3. Executor (`forge-executor`)

**Purpose**: Execute build commands and manage processes

**Architecture**:
```
Executor
    ├── ProcessManager (spawns processes)
    ├── OutputCollector (captures stdout/stderr)
    ├── TimeoutHandler (manages timeouts)
    └── SignalHandler (handles cancellation)
```

**Execution Flow**:
```
CommandSpec → Process Spawn → Output Capture → Result Collection
```

### 4. Scheduler (`forge-scheduler`)

**Purpose**: Schedule tasks for parallel execution

**Architecture**:
```
Scheduler
    ├── ReadyQueue (available tasks)
    ├── WorkerPool (execution workers)
    ├── TaskDispatcher (assigns tasks to workers)
    └── ResultHandler (processes completions)
```

**Scheduling Algorithm**:
1. Identify ready tasks (dependencies satisfied)
2. Distribute across available workers
3. Respect task priorities
4. Handle failures and retries

### 5. Cache (`forge-cache`)

**Purpose**: Fingerprint-based caching for incremental builds

**Architecture**:
```
CacheSystem
    ├── FingerprintComputer (Blake3 hashing)
    ├── CacheStore (artifact storage)
    ├── CacheValidator (validity checking)
    └── CacheEviction (LRU policy)
```

**Cache Strategy**:
- **File-level**: Cache individual file fingerprints
- **Package-level**: Cache entire package outputs
- **Hybrid**: Combine both strategies

### 6. CAS Engine (`forge-cas`)

**Purpose**: Content-Addressable Storage for artifact caching

**Architecture**:
```
CASEngine
    ├── ArtifactStore (storage interface)
    ├── CompressionEngine (Zstandard)
    ├── DeduplicationEngine (hash-based)
    └── StorageBackend (local/remote)
```

**Storage Backends**:
- **Local**: File system storage
- **Remote**: S3, GCS, MinIO
- **Composite**: Tiered L1/L2 caching

### 7. Remote Cache (`forge-remote-cache`)

**Purpose**: Tiered L1/L2 composite caching

**Architecture**:
```
RemoteCache
    ├── L1Cache (local fast cache)
    ├── L2Cache (remote shared cache)
    ├── CacheCoordinator (population/eviction)
    └── CacheMetrics (hit/miss tracking)
```

**Cache Policies**:
- **Write-through**: Write to both L1 and L2
- **Write-back**: Write to L1, async to L2
- **Read-through**: Check L1, fallback to L2

### 8. Worker (`forge-worker`)

**Purpose**: Distributed build execution

**Architecture**:
```
WorkerSystem
    ├── WorkerServer (worker daemon)
    ├── WorkerClient (client library)
    ├── ClusterExecutor (cluster management)
    └── VirtualFileSystem (on-demand file access)
```

**Worker Protocol**:
```
Client                    Worker
  |                          |
  |-- Register ------------->|
  |<-- WorkerInfo ---------|
  |                          |
  |-- SubmitTask ---------->|
  |<-- TaskResult ---------|
  |                          |
  |-- VFSRequest ---------->|
  |<-- VFSResponse --------|
```

### 9. Sandboxing (`forge-sandbox`)

**Purpose**: Hermetic environment isolation

**Architecture**:
```
SandboxSystem
    ├── Namespace (process isolation)
    ├── Filesystem (path isolation)
    ├── Network (network isolation)
    └── ResourceLimiter (CPU/memory limits)
```

**Isolation Levels**:
- **None**: No isolation (development)
- **Basic**: Filesystem isolation
- **Full**: Full isolation (production)

### 10. Plugin System (`forge-plugin`)

**Purpose**: Extensible rule system

**Architecture**:
```
PluginSystem
    ├── PluginManager (load/manage plugins)
    ├── ScriptPlugin (script-based plugins)
    ├── NativePlugin (Rust-based plugins)
    └── PluginExecutor (execution engine)
```

**Plugin Types**:
- **Shell**: Bash/sh scripts
- **Python**: Python scripts
- **Node**: Node.js scripts
- **WASM**: WebAssembly modules
- **Lua**: Lua scripts

## Language Backends

### Backend Architecture

All backends implement a common interface:

```rust
pub trait Backend {
    fn detect(&self, path: &Path) -> bool;
    fn extract_dependencies(&self, path: &Path) -> Result<Vec<Dependency>>;
    fn generate_tasks(&self, package: &Package) -> Result<Vec<Task>>;
}
```

### Backend Implementation Pattern

```
Backend
    ├── ManifestParser (read backend-specific config)
    ├── DependencyExtractor (extract dependencies)
    ├── TaskGenerator (create build tasks)
    └── Fingerprinter (backend-specific fingerprinting)
```

## Security Architecture

### 1. Artifact Signing (`forge-signing`)

**Architecture**:
```
SigningSystem
    ├── KeyManager (Ed25519 key pairs)
    ├── SignatureEngine (cryptographic signing)
    ├── SBOMGenerator (SPDX/CycloneDX)
    └── Verifier (signature verification)
```

**Signing Flow**:
```
Artifact → Hash → Sign → Signature + SBOM
```

### 2. Security Scanner (`forge-security`)

**Architecture**:
```
SecurityScanner
    ├── BackendScanners (Rust, NPM, Maven)
    ├── VulnerabilityDatabase (NVD, GitHub Advisory)
    ├── SeverityAnalyzer (CVSS scoring)
    └── PolicyEngine (blocking rules)
```

**Scanning Flow**:
```
Dependencies → Query Database → Match Vulnerabilities → Apply Policy
```

### 3. Secret Management (`forge-secrets`)

**Architecture**:
```
SecretSystem
    ├── SecretProviders (Vault, AWS, K8s)
    ├── SecretInjector (environment injection)
    ├── AuditLogger (usage tracking)
    └── AccessControl (policy enforcement)
```

## CI/CD Architecture

### 1. CI Generator (`forge-ci-generator`)

**Architecture**:
```
CIGenerator
    ├── PlatformGenerators (GitHub, GitLab, CircleCI, Bitbucket)
    ├── MatrixGenerator (test matrices)
    ├── TemplateEngine (Handlebars)
    └── ConfigValidator (validation)
```

### 2. Multi-Platform (`forge-multiplatform`)

**Architecture**:
```
MultiPlatformSystem
    ├── PlatformDetector (OS detection)
    ├── ArchitectureDetector (CPU architecture)
    ├── TargetGenerator (Rust target triples)
    └── MatrixBuilder (combination builder)
```

## Data Flow Diagrams

### Build Execution Flow

```
┌─────────────┐
│   Workspace │
│  Discovery  │
└──────┬──────┘
       │
       v
┌─────────────┐
│ Build Graph │
│ Construction│
└──────┬──────┘
       │
       v
┌─────────────┐
│   Fingerprint│
│  Computation │
└──────┬──────┘
       │
       v
┌─────────────┐
│   Scheduler  │
│   Distribution│
└──────┬──────┘
       │
       v
┌─────────────┐
│   Executor  │
│   Execution │
└──────┬──────┘
       │
       v
┌─────────────┐
│     Cache    │
│   Population │
└─────────────┘
```

### Distributed Build Flow

```
┌─────────┐    ┌─────────┐    ┌─────────┐
│ Client  │───▶│ Scheduler│───▶│  Worker1│
└─────────┘    └─────────┘    └─────────┘
                    │
                    v
              ┌─────────┐
              │  Worker2│
              └─────────┘
                    │
                    v
              ┌─────────┐
              │  WorkerN│
              └─────────┘
```

## Performance Optimizations

### 1. Level Partitioning

Groups independent packages per build level into single toolchain calls.

**Example**:
```
Level 1: [pkg-a, pkg-b, pkg-c] → single cargo build
Level 2: [pkg-d, pkg-e] → single cargo build
```

### 2. Cache-First Execution

Fingerprint-based caching enables instant rebuilds.

**Cache Key**: `hash(inputs + environment + toolchain)`

### 3. Parallel Execution

Tasks executed in parallel respecting dependencies.

**Parallelism**: Limited by CPU cores and available workers

### 4. Incremental Builds

Only rebuild affected packages based on dependency graph changes.

**Change Detection**: File content hashing + dependency analysis

## Security Considerations

### Code Safety

- `#![forbid(unsafe_code)]` in security-sensitive crates
- Input validation across all backends
- No use of unwrap() in production code

### Authentication

- Worker authentication with tokens
- Secret management with Vault/AWS/K8s
- Artifact signing verification

### Audit Logging

- Build execution logs
- Secret access logs
- Cache access logs

## Extension Points

### Custom Backends

Implement the `Backend` trait:

```rust
impl Backend for MyBackend {
    fn detect(&self, path: &Path) -> bool { /* ... */ }
    fn extract_dependencies(&self, path: &Path) -> Result<Vec<Dependency>> { /* ... */ }
    fn generate_tasks(&self, package: &Package) -> Result<Vec<Task>> { /* ... */ }
}
```

### Custom Plugins

Create plugins in `.forge/plugins/`:

```json
{
  "name": "my-plugin",
  "type": "shell",
  "main": "plugin.sh",
  "commands": {
    "build": "plugin.sh build"
  }
}
```

### Custom CI Platforms

Implement the `CIGenerator` trait:

```rust
impl CIGenerator for MyCIGenerator {
    fn generate(&self, config: &CIConfig) -> Result<String> { /* ... */ }
}
```

## Deployment Architecture

### Single Machine

```
┌─────────────────────────────────┐
│         Forge CLI               │
│  ┌─────────────────────────┐   │
│  │   Build Graph & Cache   │   │
│  └─────────────────────────┘   │
└─────────────────────────────────┘
```

### Distributed Cluster

```
┌─────────────┐    ┌─────────────┐
│  Coordinator│───▶│   Worker 1  │
└─────────────┘    └─────────────┘
       │
       ├───▶│   Worker 2  │
       │   └─────────────┘
       │
       ├───▶│   Worker 3  │
       │   └─────────────┘
       │
       └───▶│   Worker N  │
           └─────────────┘
```

### Multi-Region

```
┌─────────────┐    ┌─────────────┐
│  Coordinator│───▶│  Region 1   │
└─────────────┘    └─────────────┘
       │
       └───▶│  Region 2   │
           └─────────────┘
```

## Monitoring and Observability

### Metrics

- Build duration
- Cache hit rate
- Worker utilization
- Task queue length
- Error rates

### Logging

- Structured JSON logging
- Log levels: error, warn, info, debug, trace
- Per-component logging

### Tracing

- Distributed tracing for distributed builds
- Request/response tracking
- Performance profiling

## Future Enhancements

### Machine Learning Integration

- Predictive caching
- Build time estimation
- Resource optimization

### Advanced Visualization

- Real-time build graph visualization
- Performance heatmaps
- Dependency analysis

### Cloud-Native Features

- Kubernetes operator
- Auto-scaling workers
- Managed service offering
