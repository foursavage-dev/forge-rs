# Forge Architecture

This document describes the high-level architecture of the Forge build orchestration system.

## Overview

Forge is a cache-first, polyglot build orchestration system designed for monorepos and polyglot projects. It uses a dependency graph, parallel scheduler, executor, and CAS artifact cache to optimize build performance.

## Core Components

### 1. Workspace Discovery (`forge-core`)

**Purpose**: Discover and model the project structure

**Responsibilities**:
- Scan workspace for packages/projects
- Detect project types based on manifest files
- Filter input files by micro-globs (`MicroInputFilter`)
- Build dependency graph between packages
- Manage package metadata

**Key Types**:
- `Package`: Represents a single package/project
- `Workspace`: Collection of packages with dependencies
- `Manifest`: Project configuration (Cargo.toml, package.json, etc.)
- `MicroInputFilter`: Fine-grained glob matcher and file filter

### 2. Build Graph (`forge-graph`)

**Purpose**: Model build dependencies, execution order, and algebraic queries

**Responsibilities**:
- Create directed acyclic graph (DAG) of build tasks
- Compute topological sort for execution order
- Subgraph merging for polyglot monorepos (`merge_subgraph`)
- Dynamic node expansion during runtime execution (`DynamicGraphExpander`)
- Track task states (pending, running, completed, failed)
- Algebraic query evaluation (`GraphQueryEngine` supporting `deps()`, `rdeps()`, `allpaths()`, `somepath()`, `filter()`)
- Detect circular dependencies

**Key Types**:
- `BuildGraph`: Directed acyclic graph of tasks
- `Node`: Individual build task
- `NodeId`: Type-safe index into graph structures
- `DynamicGraphExpander`: Dynamic sub-task generator
- `GraphQueryEngine`: Evaluator for graph query expressions
- `QueryExpr`: Algebraic query AST

### 3. Executor (`forge-executor`)

**Purpose**: Execute build commands, manage processes, and handle file system cloning

**Responsibilities**:
- Spawn and manage build processes
- Capture stdout/stderr
- Handle process timeouts and cancellation
- Fast file system cloning using copy-on-write extents and hardlinks (`KernelCowCloner`)
- Fast linker auto-detection and flag synthesis (`LinkerDispatcher` supporting `mold`, `lld`, and `msvc`)
- Automatic response file synthesis (`@forge_args.rsp`) when arguments exceed OS limits
- Extensible task middleware pipeline (`TaskMiddleware`, `TurboLinker`, `SuperOptimizer`)
- Return execution results

**Key Types**:
- `CommandSpec`: Command specification with environment
- `AsyncExecutor`: Non-blocking process execution engine
- `KernelCowCloner`: Copy-on-write and fast cloner
- `LinkerDispatcher`: Modern linker detector
- `ResponseFileWriter`: Argument file synthesizer
- `TaskMiddleware`: Middleware trait for task interception
- `ExecutionResult`: Result of command execution

### 4. Scheduler (`forge-scheduler`)

**Purpose**: Schedule tasks for parallel, speculative, and distributed execution

**Responsibilities**:
- Maintain ready queue of available tasks
- Distribute tasks across available workers
- Kernel resource governor (`KernelResourceGovernor`) monitoring system memory pressure and throttling concurrency
- Compiler pipelining coordination (`PipelinedCompilationCoordinator`) unblocking downstream compilation upon metadata readiness
- GNU Jobserver pool integration (`JobserverPool`) for global thread token management across compilers
- Dynamic remote racing (`DynamicRacingExecutor`): concurrent local vs remote execution
- Distributed Task Execution (DTE) bin-packing (`DteBinPacker`) using Longest Processing Time (LPT) scheduling
- Respect task dependencies
- Handle task completion and failure

**Key Types**:
- `Scheduler`: Task scheduling engine
- `KernelResourceGovernor`: Memory pressure monitor
- `PipelinedCompilationCoordinator`: Pipelined stage manager
- `JobserverPool`: Global token-based concurrency pool
- `DynamicRacingExecutor`: Local vs remote racer
- `DteBinPacker`: Balanced multi-agent CI partitioner
- `WorkStealingPool`: Lock-free task distributor

### 5. Cache (`forge-cache`)

**Purpose**: Fingerprint-based caching for incremental builds

**Responsibilities**:
- Compute file content fingerprints (Blake3)
- Cache execution results
- Determine cache validity
- Support cache invalidation

**Key Types**:
- `Fingerprint`: Content hash with metadata
- `CacheEntry`: Cached execution result
- `FileLevelCache`: File-level caching strategy

### 6. CAS Engine (`forge-cas`)

**Purpose**: Content-Addressable Storage for artifact caching

**Responsibilities**:
- Store artifacts by content hash
- Support local and remote storage
- Compress artifacts (Zstandard)
- Provide deduplication

**Key Types**:
- `ArtifactStore`: Artifact storage interface
- `LocalStorage`: Local file system storage
- `RemoteStorage`: Remote storage (S3, GCS, MinIO)

### 7. Remote Cache (`forge-remote-cache`)

**Purpose**: Tiered L1/L2 composite caching

**Responsibilities**:
- Local L1 cache for fast access
- Remote L2 cache for sharing
- Cache population and eviction
- Cache hit/miss tracking

**Key Types**:
- `CompositeCache`: Tiered cache implementation
- `CachePolicy`: Cache population and eviction policies

### 8. Worker (`forge-worker`)

**Purpose**: Distributed build execution

**Responsibilities**:
- Remote worker discovery and registration
- Task distribution across workers
- Result collection and aggregation
- Virtual File System for on-demand file access

**Key Types**:
- `WorkerServer`: Worker daemon
- `ClusterExecutor`: Cluster task execution
- `VirtualFileSystem`: In-memory VFS

### 9. Sandboxing (`forge-sandbox`)

**Purpose**: Hermetic environment isolation

**Responsibilities**:
- Isolate build environments
- Control filesystem access
- Network isolation
- Resource limits

**Key Types**:
- `Sandbox`: Sandbox implementation
- `SandboxConfig`: Sandbox configuration

### 10. Plugin System (`forge-plugin`)

**Purpose**: Extensible rule system

**Responsibilities**:
- Load custom plugins
- Script plugin execution (Shell, Python, Node, WASM, Lua)
- Plugin discovery and management
- Plugin API

**Key Types**:
- `PluginManager`: Plugin manager
- `ScriptPlugin`: Script-based plugin
- `PluginExecutor`: Plugin execution engine

## Language Backends

Each backend implements a common interface for project detection, dependency extraction, and task generation.

### Backend Interface

```rust
pub trait Backend {
    fn detect(&self, path: &Path) -> bool;
    fn extract_dependencies(&self, path: &Path) -> Result<Vec<Dependency>>;
    fn generate_tasks(&self, package: &Package) -> Result<Vec<Task>>;
}
```

### Supported Backends

- **Rust** (`forge-backend-rust`): Cargo workspaces
- **C/C++** (`forge-backend-cc`): gcc/clang/msvc
- **Go** (`forge-backend-go`): go.mod
- **TypeScript/JS** (`forge-backend-ts`): package.json
- **Python** (`forge-backend-py`): pyproject.toml
- **Java** (`forge-backend-java`): Maven/Gradle
- **.NET** (`forge-backend-dotnet`): csproj/sln
- **Swift** (`forge-backend-swift`): Package.swift
- **Dart** (`forge-backend-dart`): pubspec.yaml
- **Zig** (`forge-backend-zig`): build.zig
- **Docker** (`forge-backend-docker`): Dockerfile

## Security Features

### 1. Artifact Signing (`forge-signing`)

- Ed25519 cryptographic signing
- SBOM generation (SPDX/CycloneDX)
- Artifact verification
- Source-to-build chain tracking

### 2. Security Scanner (`forge-security`)

- Dependency vulnerability scanning
- Multi-backend support
- Severity-based blocking
- CVSS score tracking

### 3. Secret Management (`forge-secrets`)

- HashiCorp Vault integration
- AWS Secrets Manager
- Kubernetes secrets
- Secure secret injection

## CI/CD Generation

### CI Generator (`forge-ci-generator`)

Supports multiple CI/CD platforms:
- GitHub Actions
- GitLab CI
- CircleCI
- Bitbucket Pipelines

### Matrix Generation

- Multi-platform support (Linux, macOS, Windows)
- Multi-architecture (x86_64, ARM64)
- Version matrices (Rust, Node, etc.)
- Dependency-based optimization

## Advanced Features

### 1. Build Analytics (`forge-analytics`)

- Real-time cache hit rate tracking
- Build metrics collection
- Performance visualization
- Optimization suggestions

### 2. Multi-Platform CI (`forge-multiplatform`)

- Platform detection
- Target triple generation
- Matrix configuration
- Parallel execution

### 3. Notifications (`forge-notifications`)

- Slack webhook integration
- Discord webhook support
- Email notifications
- Rich build context

### 4. Flaky Test Detection (`forge-flaky-detection`)

- Statistical analysis
- Configurable retry policies
- Test history tracking
- Failure rate monitoring

### 5. Docker Builder (`forge-docker-builder`)

- First-class Docker artifacts
- Layer caching
- Registry integration
- Multi-stage builds

### 6. Incremental Analysis (`forge-incremental`)

- AST-based dependency inference (`DependencyInferenceEngine`) for Rust, TypeScript/JavaScript, Python, and Go
- Dirty rebuild diagnostics (`DirtyExplainer`, `forge build --explain`) identifying exact source file modifications or hash mismatches
- Build pattern detection and hotspot identification
- Refactoring suggestions and rebuild frequency analysis

### 7. Build Daemon & IPC (`forge-cli::daemon`)

- Background loopback TCP daemon (`ForgeDaemon`) on `127.0.0.1:9527`
- Sub-millisecond graph caching and warm execution
- Commands: `forge daemon start`, `forge daemon status`, `forge daemon stop`

### 8. Profile-Guided Optimization (`forge-cli::pgo`)

- 2-phase LLVM PGO workflow orchestration (`PgoManager`)
- Automated `-Cprofile-generate` instrumentation and `llvm-profdata merge`
- Recompilation with `-Cprofile-use` for maximum runtime performance

### 9. Task Pipeline Topology (`forge-cli::pipeline`)

- Turborepo/Nx style topological task pipelines configured via `forge.toml`
- Cross-package dependency rules (e.g. `^build` ensuring dependency outputs are built first)
- Configurable environment variable and input file fingerprint hashes

### 10. Pipeline Templates (`forge-templates`)

- Shareable templates
- Handlebars rendering
- Template registry
- Custom templates

## Data Flow

### Build Execution Flow

```
1. Workspace Discovery
   ↓
2. Dependency Graph Construction
   ↓
3. Cache Fingerprint Computation
   ↓
4. Scheduler Task Distribution
   ↓
5. Executor Process Management
   ↓
6. Result Collection & Caching
   ↓
7. Build Completion
```

### Distributed Build Flow

```
1. Worker Registration
   ↓
2. Task Distribution
   ↓
3. VFS File Streaming
   ↓
4. Remote Execution
   ↓
5. Result Aggregation
   ↓
6. Cache Population
```

## Performance Optimizations

### 1. Level Partitioning

Groups independent packages per build level into single toolchain calls, eliminating process spawn overhead.

### 2. Cache-First Execution

Fingerprint-based caching enables instant rebuilds when inputs haven't changed.

### 3. Parallel Execution

Tasks are executed in parallel respecting dependencies, maximizing CPU utilization.

### 4. Incremental Builds

Only rebuild affected packages based on dependency graph changes.

### 5. Distributed Execution

Remote workers enable horizontal scaling for large projects.

## Security Considerations

- No unsafe code in security-sensitive crates
- Input validation across all backends
- Least privilege for all operations
- Audit logging for security operations
- Secure secret management

## Future Enhancements

- Real-time collaboration features
- Advanced visualizations
- Machine learning-based optimization
- Cloud-native deployment options
- Enhanced plugin ecosystem
