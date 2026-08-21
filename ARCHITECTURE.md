# Fish Architecture

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

This document describes the high-level architecture of the fish build orchestration system.

## Overview

Fish is a cache-first, polyglot build orchestration system designed for monorepos and polyglot projects. It uses a dependency graph, parallel scheduler, executor, and CAS artifact cache to optimize build performance.

## Core Components

### 1. Workspace Discovery (`fish-core`)

**Purpose**: Discover and model the project structure

**Responsibilities**:
- Scan workspace for packages/projects
- Detect project types based on manifest files
- Filter input files by micro-globs (`MicroInputFilter`)
- Build dependency graph between packages
- Generate IDE compilation databases (`CompilationDatabase`, `compile_commands.json`)
- Manage and isolate hermetic compiler toolchains (`ToolchainRegistry`, `ToolchainSpec`)
- Manage package metadata

**Key Types**:
- `Package`: Represents a single package/project
- `Workspace`: Collection of packages with dependencies
- `Manifest`: Project configuration (Cargo.toml, package.json, etc.)
- `MicroInputFilter`: Fine-grained glob matcher and file filter
- `CompilationDatabase`: Standard compilation command database
- `ToolchainRegistry`: Hermetic toolchain configuration manager

### 2. Build Graph (`fish-graph`)

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

### 3. Executor (`fish-executor`)

**Purpose**: Execute build commands, manage processes, and handle file system cloning

**Responsibilities**:
- Spawn and manage build processes
- Capture stdout/stderr
- Handle process timeouts and cancellation
- Fast file system cloning using copy-on-write extents and hardlinks (`KernelCowCloner`)
- Fast linker auto-detection and flag synthesis (`LinkerDispatcher` supporting `mold`, `lld`, and `msvc`)
- Automatic response file synthesis (`@fish_args.rsp`) when arguments exceed OS limits
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

### 4. Scheduler (`fish-scheduler`)

**Purpose**: Schedule tasks for parallel, speculative, and distributed execution

**Responsibilities**:
- Maintain ready queue of available tasks
- Distribute tasks across available workers
- Kernel resource governor (`KernelResourceGovernor`) monitoring system memory pressure and throttling concurrency
- Compiler pipelining coordination (`PipelinedCompilationCoordinator`) unblocking downstream compilation upon metadata readiness
- GNU Jobserver pool integration (`JobserverPool`) for global thread token management across compilers
- Dynamic remote racing (`DynamicRacingExecutor`): concurrent local vs remote execution
- Distributed Task Execution (DTE) bin-packing (`DteBinPacker`) using Longest Processing Time (LPT) scheduling
- Real-time filesystem watcher daemon (`FsWatcherDaemon`) with dirty node invalidation and hot graph cache pre-warming
- Respect task dependencies
- Handle task completion and failure

**Key Types**:
- `Scheduler`: Task scheduling engine
- `KernelResourceGovernor`: Memory pressure monitor
- `PipelinedCompilationCoordinator`: Pipelined stage manager
- `JobserverPool`: Global token-based concurrency pool
- `DynamicRacingExecutor`: Local vs remote racer
- `DteBinPacker`: Balanced multi-agent CI partitioner
- `FsWatcherDaemon`: Real-time change listener and dirty node tracker
- `WorkStealingPool`: Lock-free task distributor

### 5. Cache (`fish-cache`)

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

### 6. CAS Engine (`fish-cas`)

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

### 7. Remote Cache (`fish-remote-cache`)

**Purpose**: Tiered L1/L2 composite caching

**Responsibilities**:
- Local L1 cache for fast access
- Remote L2 cache for sharing
- Cache population and eviction
- Cache hit/miss tracking

**Key Types**:
- `CompositeCache`: Tiered cache implementation
- `CachePolicy`: Cache population and eviction policies

### 8. Worker (`fish-worker`)

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

### 9. Sandboxing (`fish-sandbox`)

**Purpose**: Hermetic environment isolation

**Responsibilities**:
- Isolate build environments
- Control filesystem access
- Network isolation
- Resource limits

**Key Types**:
- `Sandbox`: Sandbox implementation
- `SandboxConfig`: Sandbox configuration

### 10. Plugin System (`fish-plugin`)

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

- **Rust** (`fish-backend-rust`): Cargo workspaces
- **C/C++** (`fish-backend-cc`): gcc/clang/msvc
- **Go** (`fish-backend-go`): go.mod
- **TypeScript/JS** (`fish-backend-ts`): package.json
- **Python** (`fish-backend-py`): pyproject.toml
- **Java** (`fish-backend-java`): Maven/Gradle
- **.NET** (`fish-backend-dotnet`): csproj/sln
- **Swift** (`fish-backend-swift`): Package.swift
- **Dart** (`fish-backend-dart`): pubspec.yaml
- **Zig** (`fish-backend-zig`): build.zig
- **Docker** (`fish-backend-docker`): Dockerfile

## Security Features

### 1. Artifact Signing (`fish-signing`)

- Ed25519 cryptographic signing
- SBOM generation (SPDX/CycloneDX)
- Artifact verification
- Source-to-build chain tracking

### 2. Security Scanner (`fish-security`)

- Dependency vulnerability scanning
- Multi-backend support
- Severity-based blocking
- CVSS score tracking

### 3. Secret Management (`fish-secrets`)

- HashiCorp Vault integration
- AWS Secrets Manager
- Kubernetes secrets
- Secure secret injection

## CI/CD Generation

### CI Generator (`fish-ci-generator`)

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

### 1. Build Analytics (`fish-analytics`)

- Real-time cache hit rate tracking
- Build metrics collection
- Performance visualization
- Optimization suggestions

### 2. Multi-Platform CI (`fish-multiplatform`)

- Platform detection
- Target triple generation
- Matrix configuration
- Parallel execution

### 3. Notifications (`fish-notifications`)

- Slack webhook integration
- Discord webhook support
- Email notifications
- Rich build context

### 4. Flaky Test Detection (`fish-flaky-detection`)

- Statistical analysis
- Configurable retry policies
- Test history tracking
- Failure rate monitoring

### 5. Docker Builder (`fish-docker-builder`)

- First-class Docker artifacts
- Layer caching
- Registry integration
- Multi-stage builds

### 6. Incremental Analysis (`fish-incremental`)

- AST-based dependency inference (`DependencyInferenceEngine`) for Rust, TypeScript/JavaScript, Python, and Go
- Dirty rebuild diagnostics (`DirtyExplainer`, `fish build --explain`) identifying exact source file modifications or hash mismatches
- Build pattern detection and hotspot identification
- Refactoring suggestions and rebuild frequency analysis

### 7. Build Daemon & IPC (`fish-cli::daemon`)

- Background loopback TCP daemon (`FishDaemon`) on `127.0.0.1:9527`
- Sub-millisecond graph caching and warm execution
- Commands: `Fish daemon start`, `Fish daemon status`, `Fish daemon stop`

### 8. Profile-Guided Optimization (`fish-cli::pgo`)

- 2-phase LLVM PGO workflow orchestration (`PgoManager`)
- Automated `-Cprofile-generate` instrumentation and `llvm-profdata merge`
- Recompilation with `-Cprofile-use` for maximum runtime performance

### 9. Task Pipeline Topology (`fish-cli::pipeline`)

- Turborepo/Nx style topological task pipelines configured via `fish.toml`
- Cross-package dependency rules (e.g. `^build` ensuring dependency outputs are built first)
- Configurable environment variable and input file fingerprint hashes

### 10. Pipeline Templates (`fish-templates`)

- Shareable templates
- Handlebars rendering
- Template registry
- Custom templates

## Data Flow

### Build Execution Flow

```
1. Workspace Discovery
   â†“
2. Dependency Graph Construction
   â†“
3. Cache Fingerprint Computation
   â†“
4. Scheduler Task Distribution
   â†“
5. Executor Process Management
   â†“
6. Result Collection & Caching
   â†“
7. Build Completion
```

### Distributed Build Flow

```
1. Worker Registration
   â†“
2. Task Distribution
   â†“
3. VFS File Streaming
   â†“
4. Remote Execution
   â†“
5. Result Aggregation
   â†“
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

## Tri-Engine Polyglot Architecture (Rust + Python + Go)

Fish adopts a specialized tri-engine architecture where each layer is built using the language best suited for its operational requirements:

### 1. Rust Core Engine (75%) — High-Performance Execution & Local Builds
- **`fish-core`**: Workspace discovery, manifest models, fine-grained input filtering.
- **`fish-graph`**: Dependency graph, topological sort, algebraic query evaluation (`deps`, `rdeps`, `somepath`).
- **`fish-executor`**: Process execution, middleware chain, response file generation.
- **`fish-scheduler`**: GNU Jobserver pool, work-stealing, parallel execution.
- **`fish-cache`**: Multi-tier fingerprinting with Blake3 and two-phase pruning.
- **`fish-cas`**: Content-addressable artifact storage with ZSTD compression.
- **`fish-cli`**: Terminal user interface powered by ratatui and clap.

### 2. Python AI Services (15%) — Intelligence & Telemetry Layer
- **`fish_ai_analyzer`**: Machine learning build failure analysis, error classification, and root-cause suggestions.
- **`fish_optimizer`**: AI-driven DAG scheduling optimization and duration prediction.
- **`fish_analytics`**: Build run metrics aggregation, cache efficiency tracking, and bottleneck identification.
- **`fish_recommender`**: Smart build target prediction and flaky test candidate detection.

### 3. Go Services (10%) — Cloud-Native Networking & Coordination
- **`fish-coordinator`**: Distributed worker node registry, heartbeat monitoring, and optimal task dispatching.
- **`fish-worker-gateway`**: High-performance reverse proxy and streaming multiplexer for worker nodes.
- **`fish-db-migrator`**: Schema versioning and migration tooling for build telemetry databases.
- **`fish-network`**: Connection pooling, mTLS transport configuration, and high-throughput TCP streaming.

### 4. Shared Protobuf Contracts (`proto/`)
- **`proto/fish/v1/build.proto`**: Core task definitions, task results, and build graph schemas.
- **`proto/fish/v1/ai.proto`**: Failure analysis requests, scheduling optimization plans, and recommendation queries.
- **`proto/fish/v1/coordinator.proto`**: Worker registration, heartbeat protocol, and distributed task dispatching.

## Security Considerations

- No unsafe code in security-sensitive crates
- Input validation across all backends
- Least privilege for all operations
- Audit logging for security operations
- Secure secret management
- Ed25519 artifact signing and cryptographic SBOM generation
