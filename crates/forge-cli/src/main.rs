#![forbid(unsafe_code)]

mod config;
mod predictive;
mod ramdisk;
mod render;
mod semantic;
mod swarm;
mod tui;
mod watch;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use clap::{Args, Parser, Subcommand};

use forge_backend_cc::{CcBackend, CcProjectConfig};
use forge_backend_go::{GoBackend, GoProjectConfig};
use forge_backend_py::{PyBackend, PyProjectConfig};
use forge_backend_rust::{BuildMode, RustBackend};
use forge_backend_ts::{TsBackend, TsProjectConfig};
use forge_backend_java::{JavaBackend, JavaProjectConfig};
use forge_backend_dotnet::{DotnetBackend, DotnetProjectConfig};
use forge_backend_swift::{SwiftBackend, SwiftProjectConfig};
use forge_backend_dart::{DartBackend, DartProjectConfig};
use forge_backend_zig::{ZigBackend, ZigProjectConfig};
use forge_backend_docker::DockerBackend;
use forge_cache::{CachingExecutor, LocalCache};
use forge_cas::{CasStorage, CasStorageConfig, Artifact, ArtifactHash, CleanupPolicy};
use forge_ci_generator::{CIConfig, CIMatrix, CIJob, CIPlatform, GitHubActionsGenerator, GitLabCIGenerator};
use forge_core::project::Project;
use forge_executor::{ProcessExecutor, Task, TaskExecutor};
use forge_graph::BuildGraph;
use forge_plugin::{PluginBackend, PluginRulesManifest};
use forge_remote_cache::{CompositeCache, CompositeCachingExecutor, RemoteCacheServer, TcpRemoteCacheClient};
use forge_sandbox::{SandboxConfig, SandboxedExecutor};
use forge_scheduler::Scheduler;
use forge_worker::{ClusterExecutor, RemoteWorkerClient, WorkerServer};
use tui::TuiDashboard;

use crate::config::{BackendChoice, ForgeConfig};

#[derive(Debug, Parser)]
#[command(
    name = "forge",
    version = env!("CARGO_PKG_VERSION"),
    about = "🦀 Forge: a fast, cache-first build orchestration system for Rust and beyond.",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Version,
    Build(BuildArgs),
    Check(CheckArgs),
    Test(TestArgs),
    Clean(CleanArgs),
    Run(RunArgs),
    Graph(GraphArgs),
    Watch(WatchArgs),
    CacheServer(CacheServerArgs),
    Worker(WorkerArgs),
    Affected(AffectedArgs),
    Doctor,
    Cache(CacheArgs),
    Ci(CiArgs),
}

#[derive(Debug, Args)]
pub struct AffectedArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    /// The git revision (commit/tag/branch) to diff against. `HEAD` compares
    /// the working tree, an explicit revision compares the tree of that
    /// revision against the working tree.
    #[arg(long, default_value = "HEAD")]
    pub since: String,
    #[arg(long, default_value = "build", value_enum)]
    pub mode: AffectedMode,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum AffectedMode {
    Build,
    Check,
    Test,
}

impl AffectedMode {
    fn to_build_mode(self) -> BuildMode {
        match self {
            AffectedMode::Build => BuildMode::Build,
            AffectedMode::Check => BuildMode::Check,
            AffectedMode::Test => BuildMode::Test,
        }
    }
}

#[derive(Debug, Args)]
pub struct CacheArgs {
    /// Cache directory; defaults to `~/.forge/cache`.
    #[arg(long)]
    pub dir: Option<PathBuf>,
    #[command(subcommand)]
    pub command: CacheCommand,
}

#[derive(Debug, Subcommand)]
pub enum CacheCommand {
    /// Print how many fingerprints/objects the local cache holds and how
    /// much disk space they occupy.
    Stats,
    /// Delete stale fingerprints and oversized cache content.
    Prune(CachePruneArgs),
    /// CAS operations for artifact storage
    Cas(CasArgs),
}

#[derive(Debug, Args)]
pub struct CasArgs {
    #[command(subcommand)]
    pub command: CasCommand,
}

#[derive(Debug, Subcommand)]
pub enum CasCommand {
    /// Show CAS storage statistics
    Stats,
    /// Upload an artifact to CAS
    Upload {
        /// Path to the artifact file
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Artifact type (e.g., binary, library, docker-image)
        #[arg(long)]
        artifact_type: Option<String>,
        /// Source information (e.g., package name, target)
        #[arg(long)]
        source: Option<String>,
    },
    /// Download an artifact from CAS by hash
    Download {
        /// Artifact hash
        #[arg(value_name = "HASH")]
        hash: String,
        /// Output path for the downloaded artifact
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// List all artifacts in CAS
    List,
    /// Delete an artifact from CAS
    Delete {
        /// Artifact hash
        #[arg(value_name = "HASH")]
        hash: String,
    },
    /// Cleanup old artifacts from CAS
    Cleanup {
        /// Remove artifacts older than this age, e.g. `7d`, `12h`, `30m`
        #[arg(long = "older-than")]
        older_than: Option<String>,
        /// Keep CAS under this size, e.g. `10GB`, `500MB`
        #[arg(long = "max-size")]
        max_size: Option<String>,
    },
}

#[derive(Debug, Args)]
pub struct CiArgs {
    #[command(subcommand)]
    pub command: CiCommand,
}

#[derive(Debug, Subcommand)]
pub enum CiCommand {
    /// Initialize CI configuration
    Init {
        /// CI platform (github, gitlab, both)
        #[arg(long, default_value = "github")]
        platform: String,
        /// Enable cache
        #[arg(long, default_value = "true")]
        cache: bool,
        /// Remote cache URL
        #[arg(long)]
        remote_cache: Option<String>,
    },
    /// Export CI configuration
    Export {
        /// Output file path
        #[arg(short = 'o', long)]
        output: PathBuf,
        /// CI platform (github, gitlab)
        #[arg(long, default_value = "github")]
        platform: String,
    },
}

#[derive(Debug, Args)]
pub struct CachePruneArgs {
    /// Delete records older than this age, e.g. `7d`, `12h`, `30m`.
    #[arg(long = "older-than")]
    pub older_than: Option<String>,
    /// Keep the cache under this size, e.g. `10GB`, `500MB`. The oldest
    /// entries are removed first.
    #[arg(long = "max-size")]
    pub max_size: Option<String>,
}

#[derive(Debug, Args)]
struct BuildArgs {
    #[command(flatten)]
    common: CommonArgs,
}

#[derive(Debug, Args)]
struct CheckArgs {
    #[command(flatten)]
    common: CommonArgs,
}

#[derive(Debug, Args)]
struct TestArgs {
    #[command(flatten)]
    common: CommonArgs,
}

#[derive(Debug, Args, Clone)]
pub struct CommonArgs {
    pub path: Option<PathBuf>,
    #[arg(short = 'j', long = "jobs")]
    pub jobs: Option<usize>,
    #[arg(short = 'v', long)]
    pub verbose: bool,
    #[arg(long = "no-cache")]
    pub no_cache: bool,
    #[arg(long = "sandbox")]
    pub sandbox: bool,
    #[arg(long = "timeout")]
    pub timeout_secs: Option<u64>,
    #[arg(long = "profile", num_args = 0..=1, default_missing_value = "forge_trace.json")]
    pub profile: Option<PathBuf>,
    #[arg(long = "tui")]
    pub tui: bool,
    #[arg(long = "remote-cache")]
    pub remote_cache: Option<String>,
    #[arg(long = "remote-cache-token")]
    pub remote_cache_token: Option<String>,
    #[arg(long = "remote-workers", value_delimiter = ',')]
    pub remote_workers: Option<Vec<String>>,
    #[arg(long = "remote-workers-token")]
    pub remote_workers_token: Option<String>,
    /// Local cache directory; defaults to `~/.forge/cache`.
    #[arg(long = "cache-dir")]
    pub cache_dir: Option<PathBuf>,
    /// Ship a compressed snapshot of the working tree to remote workers so
    /// they can run tasks without sharing a filesystem.
    #[arg(long = "send-source")]
    pub send_source: bool,
    /// Throttle the build to `jobs / 2` concurrent workers (minimum 1)
    /// whenever the system's available memory drops below this percentage
    /// of total memory.
    #[arg(long = "ram-limit")]
    pub ram_limit: Option<u8>,
    #[arg(long = "semantic")]
    pub semantic: bool,
    #[arg(long = "ramdisk")]
    pub ramdisk: bool,
    #[arg(long = "swarm")]
    pub swarm: bool,
    #[arg(long = "reflink")]
    pub reflink: bool,
    #[arg(long = "hermetic-trace")]
    pub hermetic_trace: bool,
    #[arg(long = "swarm-compute")]
    pub swarm_compute: bool,
}

#[derive(Debug, Args)]
pub struct WatchArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    #[arg(long, default_value = "build")]
    pub mode: watch::WatchAction,
    #[arg(long, default_value = "200")]
    pub debounce: u64,
    #[arg(long)]
    pub clear: bool,
    #[arg(long, hide = true)]
    pub once: bool,
    #[arg(long)]
    pub predictive: bool,
}

#[derive(Debug, Args)]
struct CleanArgs {
    path: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct RunArgs {
    path: Option<PathBuf>,
    #[arg(short = 'p', long)]
    package: Option<String>,
    #[arg(long)]
    bin: Option<String>,
    #[arg(short = 'j', long = "jobs")]
    jobs: Option<usize>,
    #[arg(short = 'v', long)]
    verbose: bool,
    #[arg(last = true)]
    args: Vec<String>,
}

#[derive(Debug, Args)]
struct GraphArgs {
    path: Option<PathBuf>,
    #[arg(long, default_value_t = GraphFormat::Tree, value_enum)]
    format: GraphFormat,
}

#[derive(Debug, Args)]
pub struct CacheServerArgs {
    #[arg(long, default_value = "127.0.0.1:9091")]
    pub listen: String,
    #[arg(long)]
    pub auth_token: Option<String>,
    #[arg(long)]
    pub dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct WorkerArgs {
    #[arg(long, default_value = "127.0.0.1:9092")]
    pub listen: String,
    #[arg(long)]
    pub auth_token: Option<String>,
    #[arg(long, default_value = "forge-worker-node")]
    pub name: String,
    #[arg(long, default_value = "8")]
    pub max_concurrency: usize,
}

#[derive(Debug, Clone, clap::ValueEnum, Default)]
enum GraphFormat {
    #[default]
    Tree,
    Json,
    Dot,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Version => {
            println!("forge {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Command::Build(args) => run_build_mode(args.common, BuildMode::Build),
        Command::Check(args) => run_build_mode(args.common, BuildMode::Check),
        Command::Test(args) => run_build_mode(args.common, BuildMode::Test),
        Command::Clean(args) => run_clean(args.path.as_deref()),
        Command::Run(args) => run_run(args),
        Command::Graph(args) => run_graph(args),
        Command::CacheServer(args) => run_cache_server(args),
        Command::Worker(args) => run_worker(args),
        Command::Affected(args) => run_affected(args),
        Command::Doctor => run_doctor(),
        Command::Cache(args) => run_cache(args),
        Command::Watch(args) => {
            let start_dir = match resolve_start_dir(args.common.path.as_deref()) {
                Ok(dir) => dir,
                Err(message) => {
                    eprintln!("error: {message}");
                    return ExitCode::FAILURE;
                }
            };
            watch::run_watch(
                args.common,
                args.mode,
                args.debounce,
                args.clear,
                &start_dir,
                args.once,
                args.predictive,
            )
        }
        Command::Ci(args) => run_ci(args),
    }
}

fn run_cache_server(args: CacheServerArgs) -> ExitCode {
    println!("🦀 Forge Remote Cache Server");
    println!("Listening on: {}", args.listen);
    if let Some(dir) = &args.dir {
        println!("Storage dir:  {}", dir.display());
    }
    let server = RemoteCacheServer::new(args.listen, args.auth_token, args.dir);
    match server.run_blocking() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: cache server failed: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run_affected(args: AffectedArgs) -> ExitCode {
    let mode = args.mode.to_build_mode();
    let start_dir = match resolve_start_dir(args.common.path.as_deref()) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let config = match ForgeConfig::load(&start_dir) {
        Ok(Some(config)) => config,
        Ok(None) => ForgeConfig::default(),
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };
    if !matches!(config.backend, BackendChoice::Auto | BackendChoice::Rust) {
        eprintln!(
            "error: `forge affected` only supports Rust workspaces (backend `{:?}` configured)",
            config.backend
        );
        return ExitCode::FAILURE;
    }

    let project = match Project::discover(&start_dir) {
        Ok(Some(project)) => project,
        Ok(None) => {
            eprintln!(
                "error: no Cargo project found in `{}` or any parent directory",
                start_dir.display()
            );
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("hint: make sure `cargo` is installed and available on PATH");
            return ExitCode::FAILURE;
        }
    };

    let repo_root = match git_output(&start_dir, &["rev-parse", "--show-toplevel"]) {
        Ok(root) => PathBuf::from(root.trim()),
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let mut changed = Vec::new();
    if let Err(message) = collect_git_changes(&repo_root, &args.since, &mut changed) {
        eprintln!("error: {message}");
        return ExitCode::FAILURE;
    }

    if changed.is_empty() {
        println!("No changes since `{}`; nothing to do.", args.since);
        return ExitCode::SUCCESS;
    }

    let paths: Vec<PathBuf> = changed
        .iter()
        .map(|p| {
            let full = repo_root.join(p);
            std::fs::canonicalize(&full).unwrap_or(full)
        })
        .collect();
    let path_refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();

    let package_graph = match if mode == BuildMode::Test {
        project.build_test_graph()
    } else {
        project.build_graph()
    } {
        Ok(graph) => graph,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let owner_ids: Vec<forge_graph::NodeId> = match project.packages_for_paths(&path_refs) {
        Some(owners) => package_graph
            .nodes()
            .iter()
            .filter(|node| owners.iter().any(|owner| &node.payload == owner))
            .map(|node| node.id)
            .collect(),
        None => {
            // A file outside every package (workspace Cargo.toml, lockfile,
            // forge.toml, ...) can affect anything.
            if !args.common.tui {
                println!("Workspace-level file changed; all packages are affected.");
            }
            package_graph.nodes().iter().map(|node| node.id).collect()
        }
    };

    let affected = package_graph.affected_nodes(&owner_ids);
    if affected.is_empty() {
        println!("No packages affected by changes since `{}`.", args.since);
        return ExitCode::SUCCESS;
    }

    if !args.common.tui {
        println!(
            "Affected packages ({} of {}):",
            affected.len(),
            package_graph.len()
        );
        for id in &affected {
            if let Some(package) = package_graph.node(*id) {
                if let Some(pkg) = project.package(&package.payload) {
                    println!("  - {}", pkg.name);
                }
            }
        }
        println!();
    }

    let filtered = package_graph.subgraph(&affected);
    run_build_mode_with(args.common, mode, Some(filtered))
}

fn run_doctor() -> ExitCode {
    println!("🦀 Forge Doctor");
    let mut all_ok = true;

    for (tool, version_args) in [
        ("cargo", &["--version"][..]),
        ("git", &["--version"][..]),
    ] {
        match std::process::Command::new(tool).args(version_args).output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                println!("  [ok] {tool} found{version}", version = if version.is_empty() { String::new() } else { format!(": {version}") });
            }
            _ => {
                println!("  [fail] {tool} is not available on PATH");
                all_ok = false;
            }
        }
    }

    match LocalCache::default_location() {
        Ok(cache) => {
            let probe = cache.root().join(".doctor-probe");
            match std::fs::write(&probe, b"ok") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&probe);
                    let stats = cache.disk_stats();
                    println!(
                        "  [ok] cache dir {} is writable ({} records, {} objects, {})",
                        cache.root().display(),
                        stats.record_count,
                        stats.object_count,
                        human_bytes(stats.total_bytes)
                    );
                }
                Err(error) => {
                    println!(
                        "  [fail] cache dir {} is not writable: {error}",
                        cache.root().display()
                    );
                    all_ok = false;
                }
            }
        }
        Err(error) => {
            println!("  [fail] cannot open the local cache: {error}");
            all_ok = false;
        }
    }

    if all_ok {
        println!("All checks passed.");
        ExitCode::SUCCESS
    } else {
        println!("Some checks failed.");
        ExitCode::FAILURE
    }
}

fn run_cache(args: CacheArgs) -> ExitCode {
    let cache = match &args.dir {
        Some(dir) => match LocalCache::new(dir) {
            Ok(cache) => cache,
            Err(error) => {
                eprintln!("error: cannot open cache at `{}`: {error}", dir.display());
                return ExitCode::FAILURE;
            }
        },
        None => match LocalCache::default_location() {
            Ok(cache) => cache,
            Err(error) => {
                eprintln!("error: cannot open the default cache: {error}");
                return ExitCode::FAILURE;
            }
        },
    };

    match args.command {
        CacheCommand::Stats => {
            let stats = cache.disk_stats();
            println!("Cache dir:          {}", cache.root().display());
            println!(
                "Fingerprint records: {} ({})",
                stats.record_count,
                human_bytes(stats.fingerprints_bytes)
            );
            println!(
                "Artifact objects:    {} ({})",
                stats.object_count,
                human_bytes(stats.objects_bytes)
            );
            println!("Total:               {}", human_bytes(stats.total_bytes));
            ExitCode::SUCCESS
        }
        CacheCommand::Prune(prune) => {
            let older_than = match prune.older_than.as_deref().map(parse_duration).transpose() {
                Ok(value) => value,
                Err(message) => {
                    eprintln!("error: --older-than: {message}");
                    return ExitCode::FAILURE;
                }
            };
            let max_size = match prune.max_size.as_deref().map(parse_size).transpose() {
                Ok(value) => value,
                Err(message) => {
                    eprintln!("error: --max-size: {message}");
                    return ExitCode::FAILURE;
                }
            };
            match cache.prune(older_than, max_size) {
                Ok(report) => {
                    println!(
                        "Removed {} fingerprint records and {} objects (freed {}).",
                        report.removed_records,
                        report.removed_objects,
                        human_bytes(report.freed_bytes)
                    );
                    let stats = cache.disk_stats();
                    println!(
                        "Cache now: {} records, {} objects, {} total.",
                        stats.record_count,
                        stats.object_count,
                        human_bytes(stats.total_bytes)
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("error: prune failed: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        CacheCommand::Cas(cas_args) => {
            run_cas(&cache, cas_args)
        }
    }
}

fn run_cas(cache: &LocalCache, args: CasArgs) -> ExitCode {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    match args.command {
        CasCommand::Stats => {
            let cas_path = cache.cas_path();
            let config = CasStorageConfig::local(&cas_path);
            match rt.block_on(CasStorage::new(config)) {
                Ok(storage) => {
                    match rt.block_on(storage.stats()) {
                        Ok(stats) => {
                            println!("CAS Storage:         {}", cas_path.display());
                            println!("Backend type:        {}", stats.backend_type);
                            println!("Artifacts:          {}", stats.artifact_count);
                            println!("Total size:          {}", human_bytes(stats.total_bytes));
                            println!("Compressed size:    {}", human_bytes(stats.compressed_bytes));
                            if stats.total_bytes > 0 {
                                let ratio = stats.compressed_bytes as f64 / stats.total_bytes as f64;
                                println!("Compression ratio:   {:.2}%", (1.0 - ratio) * 100.0);
                            }
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("error: failed to get CAS stats: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: failed to initialize CAS storage: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        CasCommand::Upload { file, artifact_type, source } => {
            let artifact_type = artifact_type.unwrap_or_else(|| {
                file.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("binary")
                    .to_string()
            });
            let source = source.unwrap_or_else(|| {
                file.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });
            
            match rt.block_on(Artifact::from_file(&file)) {
                Ok(mut artifact) => {
                    artifact.metadata.artifact_type = artifact_type;
                    artifact.metadata.source = source;
                    
                    let cas_path = cache.cas_path();
                    let config = CasStorageConfig::local(&cas_path);
                    match rt.block_on(CasStorage::new(config)) {
                        Ok(storage) => {
                            match rt.block_on(storage.store(&artifact)) {
                                Ok(_) => {
                                    println!("Artifact uploaded successfully");
                                    println!("Hash: {}", artifact.hash());
                                    println!("Size: {}", human_bytes(artifact.size()));
                                    if let Some(ratio) = artifact.compression_ratio() {
                                        println!("Compression: {:.2}%", (1.0 - ratio) * 100.0);
                                    }
                                    ExitCode::SUCCESS
                                }
                                Err(e) => {
                                    eprintln!("error: failed to store artifact: {}", e);
                                    ExitCode::FAILURE
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("error: failed to initialize CAS storage: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: failed to read artifact file: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        CasCommand::Download { hash, output } => {
            let artifact_hash = ArtifactHash::new(hash.clone());
            let output_path = output.unwrap_or_else(|| {
                // Use hash as filename if no output specified
                PathBuf::from(hash)
            });
            
            let cas_path = cache.cas_path();
            let config = CasStorageConfig::local(&cas_path);
            match rt.block_on(CasStorage::new(config)) {
                Ok(storage) => {
                    match rt.block_on(storage.retrieve(&artifact_hash)) {
                        Ok(artifact) => {
                            match std::fs::write(&output_path, artifact.data()) {
                                Ok(_) => {
                                    println!("Artifact downloaded successfully");
                                    println!("Output: {}", output_path.display());
                                    println!("Size: {}", human_bytes(artifact.size()));
                                    ExitCode::SUCCESS
                                }
                                Err(e) => {
                                    eprintln!("error: failed to write artifact: {}", e);
                                    ExitCode::FAILURE
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("error: failed to retrieve artifact: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: failed to initialize CAS storage: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        CasCommand::List => {
            let cas_path = cache.cas_path();
            let config = CasStorageConfig::local(&cas_path);
            match rt.block_on(CasStorage::new(config)) {
                Ok(storage) => {
                    match rt.block_on(storage.list()) {
                        Ok(hashes) => {
                            println!("CAS Artifacts ({} total):", hashes.len());
                            for hash in hashes {
                                println!("  {}", hash);
                            }
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("error: failed to list artifacts: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: failed to initialize CAS storage: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        CasCommand::Delete { hash } => {
            let artifact_hash = ArtifactHash::new(hash);
            let cas_path = cache.cas_path();
            let config = CasStorageConfig::local(&cas_path);
            match rt.block_on(CasStorage::new(config)) {
                Ok(storage) => {
                    match rt.block_on(storage.delete(&artifact_hash)) {
                        Ok(_) => {
                            println!("Artifact deleted successfully");
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("error: failed to delete artifact: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: failed to initialize CAS storage: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        CasCommand::Cleanup { older_than, max_size } => {
            let older_than_duration = match older_than.as_deref().map(parse_duration).transpose() {
                Ok(value) => value,
                Err(message) => {
                    eprintln!("error: --older-than: {message}");
                    return ExitCode::FAILURE;
                }
            };
            let max_size_bytes = match max_size.as_deref().map(parse_size).transpose() {
                Ok(value) => value,
                Err(message) => {
                    eprintln!("error: --max-size: {message}");
                    return ExitCode::FAILURE;
                }
            };
            
            let cas_path = cache.cas_path();
            let config = CasStorageConfig::local(&cas_path).with_max_size(max_size_bytes.unwrap_or(0));
            match rt.block_on(CasStorage::new(config)) {
                Ok(storage) => {
                    let policy = if let Some(duration) = older_than_duration {
                        CleanupPolicy::OlderThan(duration)
                    } else {
                        CleanupPolicy::OlderThan(std::time::Duration::from_secs(7 * 24 * 60 * 60)) // 7 days default
                    };
                    
                    match rt.block_on(storage.cleanup(policy)) {
                        Ok(result) => {
                            println!("Removed {} artifacts", result.removed_count);
                            println!("Freed {}", human_bytes(result.freed_bytes));
                            if let Some(max_bytes) = max_size_bytes {
                                println!("Max size limit: {}", human_bytes(max_bytes));
                            }
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("error: failed to cleanup CAS: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: failed to initialize CAS storage: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
    }
}

fn run_ci(args: CiArgs) -> ExitCode {
    match args.command {
        CiCommand::Init { platform, cache, remote_cache } => {
            let ci_config = CIConfig {
                platform: match platform.as_str() {
                    "github" => CIPlatform::GitHubActions,
                    "gitlab" => CIPlatform::GitLabCI,
                    "both" => CIPlatform::Both,
                    _ => {
                        eprintln!("error: invalid platform '{}', expected 'github', 'gitlab', or 'both'", platform);
                        return ExitCode::FAILURE;
                    }
                },
                cache_enabled: cache,
                remote_cache_url: remote_cache.clone(),
                jobs_per_run: 4,
                timeout_minutes: 30,
            };
            
            // Create a sample CI matrix
            let mut matrix = CIMatrix::new();
            
            // Add sample jobs for demonstration
            matrix.add_job(CIJob {
                id: "build".to_string(),
                name: "Build".to_string(),
                backend: "rust".to_string(),
                commands: vec!["cargo build --release".to_string()],
                artifacts: vec!["target/release/my_app".to_string()],
                dependencies: vec![],
                cache_key: "build-cache".to_string(),
            });
            
            matrix.add_job(CIJob {
                id: "test".to_string(),
                name: "Test".to_string(),
                backend: "rust".to_string(),
                commands: vec!["cargo test".to_string()],
                artifacts: vec![],
                dependencies: vec!["build".to_string()],
                cache_key: "test-cache".to_string(),
            });
            
            matrix.cache_config.enabled = cache;
            matrix.cache_config.remote_url = remote_cache.clone();
            
            // Generate CI configuration files
            if ci_config.platform == CIPlatform::GitHubActions || 
               ci_config.platform == CIPlatform::Both {
                let generator = GitHubActionsGenerator::new(ci_config.clone());
                match generator.generate_workflow(&matrix) {
                    Ok(workflow) => {
                        std::fs::create_dir_all(".github/workflows").ok();
                        match std::fs::write(".github/workflows/forge.yml", workflow) {
                            Ok(_) => println!("✓ Created .github/workflows/forge.yml"),
                            Err(e) => {
                                eprintln!("error: failed to write GitHub Actions workflow: {}", e);
                                return ExitCode::FAILURE;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("error: failed to generate GitHub Actions workflow: {}", e);
                        return ExitCode::FAILURE;
                    }
                }
            }
            
            if ci_config.platform == CIPlatform::GitLabCI || 
               ci_config.platform == CIPlatform::Both {
                let generator = GitLabCIGenerator::new(ci_config.clone());
                match generator.generate_pipeline(&matrix) {
                    Ok(pipeline) => {
                        match std::fs::write(".gitlab-ci.yml", pipeline) {
                            Ok(_) => println!("✓ Created .gitlab-ci.yml"),
                            Err(e) => {
                                eprintln!("error: failed to write GitLab CI pipeline: {}", e);
                                return ExitCode::FAILURE;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("error: failed to generate GitLab CI pipeline: {}", e);
                        return ExitCode::FAILURE;
                    }
                }
            }
            
            println!("✓ CI configuration initialized successfully");
            println!("  Platform: {}", platform);
            println!("  Cache: {}", if cache { "enabled" } else { "disabled" });
            if let Some(url) = &remote_cache {
                println!("  Remote cache: {}", url);
            }
            
            ExitCode::SUCCESS
        }
        CiCommand::Export { output, platform } => {
            let ci_config = CIConfig {
                platform: match platform.as_str() {
                    "github" => CIPlatform::GitHubActions,
                    "gitlab" => CIPlatform::GitLabCI,
                    _ => {
                        eprintln!("error: invalid platform '{}', expected 'github' or 'gitlab'", platform);
                        return ExitCode::FAILURE;
                    }
                },
                cache_enabled: true,
                remote_cache_url: None,
                jobs_per_run: 4,
                timeout_minutes: 30,
            };
            
            // Create a sample matrix for export
            let mut matrix = CIMatrix::new();
            matrix.add_job(CIJob {
                id: "build".to_string(),
                name: "Build".to_string(),
                backend: "rust".to_string(),
                commands: vec!["cargo build".to_string()],
                artifacts: vec![],
                dependencies: vec![],
                cache_key: "cache-key".to_string(),
            });
            
            let result = match platform.as_str() {
                "github" => {
                    let generator = GitHubActionsGenerator::new(ci_config);
                    generator.generate_workflow(&matrix)
                }
                "gitlab" => {
                    let generator = GitLabCIGenerator::new(ci_config);
                    generator.generate_pipeline(&matrix)
                }
                _ => unreachable!(),
            };
            
            match result {
                Ok(content) => {
                    match std::fs::write(&output, content) {
                        Ok(_) => {
                            println!("✓ Exported CI configuration to {}", output.display());
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("error: failed to write to {}: {}", output.display(), e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: failed to generate CI configuration: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
    }
}

/// Parses durations like `30m`, `2h`, `7d` (bare numbers are seconds).
fn parse_duration(text: &str) -> Result<std::time::Duration, String> {
    let text = text.trim();
    let (num, unit) = match text.chars().last() {
        Some(c) if c.is_ascii_digit() => (text, ""),
        _ => (&text[..text.len() - 1], &text[text.len() - 1..]),
    };
    let value: u64 = num
        .trim()
        .parse()
        .map_err(|_| format!("invalid duration `{text}` (expected e.g. `7d`, `12h`, `30m`)"))?;
    let seconds = match unit {
        "" | "s" => value,
        "m" => value * 60,
        "h" => value * 3600,
        "d" => value * 86400,
        _ => {
            return Err(format!(
                "unknown duration unit `{unit}` in `{text}` (expected s, m, h, or d)"
            ));
        }
    };
    Ok(std::time::Duration::from_secs(seconds))
}

/// Parses sizes like `500MB`, `2GB`, `10KB` (binary multiples).
fn parse_size(text: &str) -> Result<u64, String> {
    let mut text = text.trim().to_uppercase();
    if text.ends_with('B') {
        text.pop();
    }
    let (num, unit) = match text.chars().last() {
        Some(c) if c.is_ascii_digit() => (text.as_str(), "B"),
        _ => (&text[..text.len() - 1], &text[text.len() - 1..]),
    };
    let value: u64 = num
        .trim()
        .parse()
        .map_err(|_| format!("invalid size `{text}` (expected e.g. `10GB`, `500MB`)"))?;
    let factor = match unit {
        "B" => 1u64,
        "K" => 1024,
        "M" => 1024 * 1024,
        "G" => 1024 * 1024 * 1024,
        _ => {
            return Err(format!(
                "unknown size unit `{unit}` in `{text}` (expected B, KB, MB, or GB)"
            ));
        }
    };
    Ok(value * factor)
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Runs `git` with the given arguments inside `dir` and returns stdout.
fn git_output(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|error| format!("failed to run `git`: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "`git {}` exited with {}: {}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Collects the files changed since `since` (tracked diffs plus untracked
/// files), as paths relative to the repository root.
fn collect_git_changes(root: &Path, since: &str, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let diff = git_output(root, &["diff", "--name-only", since])?;
    for line in diff.lines() {
        let line = line.trim();
        if !line.is_empty() {
            out.push(PathBuf::from(line));
        }
    }
    let untracked = git_output(root, &["ls-files", "--others", "--exclude-standard"])?;
    for line in untracked.lines() {
        let line = line.trim();
        if !line.is_empty() {
            out.push(PathBuf::from(line));
        }
    }
    Ok(())
}

fn run_worker(args: WorkerArgs) -> ExitCode {
    println!("🦀 Forge Distributed Worker Node");
    println!("Worker name:  {}", args.name);
    println!("Listening on: {}", args.listen);
    println!("Concurrency:  {}", args.max_concurrency);
    let server = WorkerServer::with_options(args.listen, args.auth_token, args.name, args.max_concurrency);
    match server.run_blocking() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: worker daemon failed: {err}");
            ExitCode::FAILURE
        }
    }
}

fn build_executor(args: &CommonArgs, cache: Option<LocalCache>) -> Box<dyn TaskExecutor> {
    let local_process = ProcessExecutor::with_timeout(
        args.verbose,
        args.timeout_secs.map(std::time::Duration::from_secs),
    );

    let base_executor: Box<dyn TaskExecutor> = if let Some(workers) = &args.remote_workers {
        let clients: Vec<RemoteWorkerClient> = workers
            .iter()
            .map(|addr| RemoteWorkerClient::new(addr, args.remote_workers_token.clone()))
            .collect();
        let mut cluster = if args.sandbox {
            let sb_config = SandboxConfig::default();
            let sandboxed_local = Arc::new(SandboxedExecutor::new(local_process, sb_config));
            ClusterExecutor::with_local_fallback(clients, sandboxed_local)
        } else {
            ClusterExecutor::with_local_fallback(clients, Arc::new(local_process))
        };
        if args.send_source {
            cluster = cluster.with_source_packaging();
        }
        Box::new(cluster)
    } else if args.sandbox {
        let sb_config = SandboxConfig::default();
        Box::new(SandboxedExecutor::new(local_process, sb_config))
    } else {
        Box::new(local_process)
    };

    if let Some(local_c) = cache {
        if let Some(remote_addr) = &args.remote_cache {
            let remote_client = TcpRemoteCacheClient::new(remote_addr, args.remote_cache_token.clone());
            let composite = CompositeCache::new(local_c, Some(Box::new(remote_client)));
            Box::new(CompositeCachingExecutor::new(base_executor, composite))
        } else {
            Box::new(CachingExecutor::new(base_executor, local_c))
        }
    } else {
        base_executor
    }
}

pub(crate) fn run_build_mode(args: CommonArgs, mode: BuildMode) -> ExitCode {
    run_build_mode_with(args, mode, None)
}

/// Entry point shared by `build`/`check`/`test` and `affected`. When
/// `filtered_graph` is provided (affected builds) it replaces the
/// discovered package graph, restricting tasks to the affected packages.
fn run_build_mode_with(
    args: CommonArgs,
    mode: BuildMode,
    filtered_graph: Option<BuildGraph<cargo_metadata::PackageId>>,
) -> ExitCode {
    let start_dir = match resolve_start_dir(args.path.as_deref()) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let config = match ForgeConfig::load(&start_dir) {
        Ok(Some(config)) => config,
        Ok(None) => ForgeConfig::default(),
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let merged = CommonArgs {
        path: args.path,
        jobs: args.jobs.or_else(|| config.jobs.filter(|&j| j > 0)),
        verbose: args.verbose,
        no_cache: args.no_cache || config.no_cache,
        sandbox: args.sandbox || config.sandbox,
        timeout_secs: args.timeout_secs.or(config.timeout),
        profile: args.profile.or_else(|| config.profile.map(PathBuf::from)),
        tui: args.tui || config.tui,
        remote_cache: args.remote_cache.or(config.remote_cache),
        remote_cache_token: args.remote_cache_token.or(config.remote_cache_token),
        remote_workers: args.remote_workers.or(config.remote_workers),
        remote_workers_token: args.remote_workers_token.or(config.remote_workers_token),
        cache_dir: args.cache_dir.or_else(|| config.cache_dir.map(PathBuf::from)),
        send_source: args.send_source || config.send_source,
        ram_limit: args.ram_limit.or(config.ram_limit),
        semantic: args.semantic || config.semantic,
        ramdisk: args.ramdisk || config.ramdisk,
        swarm: args.swarm || config.swarm,
        reflink: args.reflink || config.reflink,
        hermetic_trace: args.hermetic_trace || config.hermetic_trace,
        swarm_compute: args.swarm_compute || config.swarm_compute,
    };

    if merged.ramdisk {
        if let Ok(rd) = ramdisk::RamDisk::create_turbo_workspace("turbo") {
            println!("⚡ In-memory RAM disk turbo enabled: {}", rd.path().display());
        }
    }

    if merged.swarm || merged.swarm_compute {
        let swarm_cache = swarm::SwarmCache::new(true);
        let _ = swarm_cache.broadcast_presence(7890);
        if merged.swarm_compute {
            let _ = swarm_cache.broadcast_compute_worker(7891, 4);
            let workers = swarm_cache.discovered_compute_endpoints();
            println!("🪐 Distributed P2P Compute Swarm active (LAN workers: {})", workers.len());
        } else {
            let peer_count = swarm_cache.active_peer_count();
            println!("🌐 P2P Swarm Cache enabled (active LAN peers: {})", peer_count);
        }
    }

    if merged.reflink {
        println!("⚡ Reflink / Copy-on-Write hardware VFS engine active");
    }

    if merged.hermetic_trace {
        println!("🔮 Hermetic Syscall tracing sandbox active");
    }

    if merged.semantic {
        println!("🧠 Semantic AST-aware fingerprinting active");
    }

    match config.backend {
        BackendChoice::Cc => return run_cc_build(&start_dir, &merged),
        BackendChoice::Go => return run_go_build(&start_dir, &merged),
        BackendChoice::Ts
        | BackendChoice::Typescript
        | BackendChoice::Javascript
        | BackendChoice::Js => return run_ts_build(&start_dir, &merged),
        BackendChoice::Py | BackendChoice::Python => return run_py_build(&start_dir, &merged),
        BackendChoice::Java | BackendChoice::Kotlin => return run_java_build(&start_dir, &merged),
        BackendChoice::Dotnet | BackendChoice::CSharp | BackendChoice::FSharp => return run_dotnet_build(&start_dir, &merged),
        BackendChoice::Swift | BackendChoice::ObjC | BackendChoice::ObjectiveC => return run_swift_build(&start_dir, &merged),
        BackendChoice::Dart | BackendChoice::Flutter => return run_dart_build(&start_dir, &merged),
        BackendChoice::Zig => return run_zig_build(&start_dir, &merged),
        BackendChoice::Docker | BackendChoice::Oci => return run_docker_build(&start_dir, &merged),
        BackendChoice::Plugin | BackendChoice::Rules => {
            return run_plugin_build(&start_dir, &merged);
        }
        BackendChoice::Rust => return run_rust_build(&start_dir, &merged, mode, filtered_graph),
        BackendChoice::Auto => {}
    }

    if start_dir.join("Forgefile.json").exists() || start_dir.join("forge.rules.json").exists() {
        return run_plugin_build(&start_dir, &merged);
    }

    if start_dir.join("forge.cc.json").exists() {
        return run_cc_build(&start_dir, &merged);
    }

    if start_dir.join("forge.go.json").exists()
        || (start_dir.join("go.mod").exists() && !start_dir.join("Cargo.toml").exists())
    {
        return run_go_build(&start_dir, &merged);
    }

    if start_dir.join("forge.ts.json").exists()
        || (start_dir.join("package.json").exists() && !start_dir.join("Cargo.toml").exists())
    {
        return run_ts_build(&start_dir, &merged);
    }

    if start_dir.join("forge.py.json").exists()
        || (start_dir.join("pyproject.toml").exists() && !start_dir.join("Cargo.toml").exists())
    {
        return run_py_build(&start_dir, &merged);
    }

    // Java/Kotlin detection
    if start_dir.join("pom.xml").exists()
        || start_dir.join("build.gradle").exists()
        || start_dir.join("build.gradle.kts").exists()
    {
        return run_java_build(&start_dir, &merged);
    }

    // .NET detection
    let has_dotnet_project = has_file_with_extension(&start_dir, &["csproj", "sln"]);
    if has_dotnet_project {
        return run_dotnet_build(&start_dir, &merged);
    }

    // Swift/Objective-C detection
    let has_swift_project = start_dir.join("Package.swift").exists() 
        || has_dir_with_extension(&start_dir, &["xcodeproj"]);
    if has_swift_project {
        return run_swift_build(&start_dir, &merged);
    }

    // Dart/Flutter detection
    if start_dir.join("pubspec.yaml").exists() {
        return run_dart_build(&start_dir, &merged);
    }

    // Zig detection
    if start_dir.join("build.zig").exists() {
        return run_zig_build(&start_dir, &merged);
    }

    // Docker/OCI detection
    if start_dir.join("Dockerfile").exists()
        || start_dir.join("docker-compose.yml").exists()
        || start_dir.join("docker-compose.yaml").exists()
    {
        return run_docker_build(&start_dir, &merged);
    }

    run_rust_build(&start_dir, &merged, mode, filtered_graph)
}

fn run_rust_build(
    start_dir: &Path,
    args: &CommonArgs,
    mode: BuildMode,
    filtered_graph: Option<BuildGraph<cargo_metadata::PackageId>>,
) -> ExitCode {
    let project = match Project::discover(start_dir) {
        Ok(Some(project)) => project,
        Ok(None) => {
            eprintln!(
                "error: no Cargo, C/C++, Go, TypeScript, Python, or Custom Rules project found in `{}` or any parent directory",
                start_dir.display()
            );
            eprintln!(
                "hint: run `forge build` from a directory containing Cargo.toml, forge.cc.json, go.mod, package.json, pyproject.toml, or Forgefile.json"
            );
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("hint: make sure `cargo` is installed and available on PATH");
            return ExitCode::FAILURE;
        }
    };

    let is_filtered = filtered_graph.is_some();
    let package_graph = match match filtered_graph {
        Some(filtered) => Ok(filtered),
        None if mode == BuildMode::Test => project.build_test_graph(),
        None => project.build_graph(),
    } {
        Ok(graph) => graph,
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("hint: run `cargo metadata` to inspect the workspace state");
            return ExitCode::FAILURE;
        }
    };

    let backend = match RustBackend::new() {
        Ok(backend) => backend,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut task_graph = match backend.create_tasks(&project, &package_graph, mode, !args.no_cache)
    {
        Ok(graph) => graph,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let cache = open_cache(args);
    let cache_handle = cache.clone();
    let executor = build_executor(args, cache);

    let workers = args.jobs.unwrap_or_else(default_jobs).max(1);
    let scheduler = make_scheduler(workers, args);

    if !args.tui && !is_filtered {
        render::print_project(&project, &package_graph);
        println!();
        println!("{}...", mode_verb(mode));
        println!();
    }

    let summary = if args.tui {
        let mut dashboard = TuiDashboard::new(task_graph.len());
        let _ = dashboard.start();
        let run_res = scheduler.run(&mut task_graph, &executor, |task, outcome| {
            dashboard.on_task_finish(&task.label, outcome);
        });
        let summary = match run_res {
            Ok(s) => s,
            Err(err) => {
                let _ = dashboard.finish(&forge_scheduler::BuildSummary {
                    total: task_graph.len(),
                    executed: 0,
                    cached: 0,
                    failed: 1,
                    cancelled: 0,
                    duration: std::time::Duration::from_millis(0),
                    workers,
                    failures: vec![],
                    timings: vec![],
                });
                eprintln!("error: scheduler failed: {err}");
                return ExitCode::FAILURE;
            }
        };
        let _ = dashboard.finish(&summary);
        summary
    } else {
        match scheduler.run(&mut task_graph, &executor, |task, outcome| {
            render::print_progress(task, outcome)
        }) {
            Ok(summary) => summary,
            Err(error) => {
                eprintln!("error: scheduler failed: {error}");
                return ExitCode::FAILURE;
            }
        }
    };

    render::print_build_summary(&summary, mode);
    if let Some(ref c) = cache_handle {
        render::print_cache_stats(c);
    }
    if let Some(ref trace_path) = args.profile {
        if let Err(err) = summary.write_chrome_trace(trace_path) {
            eprintln!("warning: failed to write profile trace: {err}");
        } else {
            render::print_profile_saved(trace_path);
        }
    }

    if summary.succeeded() {
        ExitCode::SUCCESS
    } else {
        render::print_failures(&summary);
        ExitCode::FAILURE
    }
}

fn run_clean(path: Option<&Path>) -> ExitCode {
    let start_dir = match resolve_start_dir(path) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let project = match Project::discover(&start_dir) {
        Ok(Some(project)) => project,
        Ok(None) => {
            eprintln!(
                "error: no Cargo project found in `{}` or any parent directory",
                start_dir.display()
            );
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("hint: make sure `cargo` is installed and available on PATH");
            return ExitCode::FAILURE;
        }
    };

    let workspace_root = project.workspace_root().to_path_buf();
    println!(
        "Cleaning: {}",
        project.workspace_root().as_std_path().display()
    );
    match std::process::Command::new("cargo")
        .arg("clean")
        .current_dir(workspace_root)
        .status()
    {
        Ok(status) if status.success() => {
            println!("Cleaned.");
            ExitCode::SUCCESS
        }
        Ok(status) => {
            eprintln!("error: `cargo clean` failed with {status}");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("error: failed to run `cargo clean`: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_run(args: RunArgs) -> ExitCode {
    let start_dir = match resolve_start_dir(args.path.as_deref()) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let project = match Project::discover(&start_dir) {
        Ok(Some(project)) => project,
        Ok(None) => {
            eprintln!(
                "error: no Cargo project found in `{}` or any parent directory",
                start_dir.display()
            );
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut target_pkg = None;
    if let Some(pkg_name) = &args.package {
        for id in project.workspace_members() {
            if let Some(pkg) = project.package(id) {
                if pkg.name.as_str() == *pkg_name {
                    target_pkg = Some(pkg);
                    break;
                }
            }
        }
        if target_pkg.is_none() {
            eprintln!("error: package `{}` not found in workspace", pkg_name);
            return ExitCode::FAILURE;
        }
    } else if let Some(root_pkg) = project.root_package() {
        target_pkg = Some(root_pkg);
    } else {
        eprintln!("error: workspace has no root package; specify one with `--package`");
        return ExitCode::FAILURE;
    }

    let pkg = target_pkg.unwrap();
    let package_name = pkg.name.to_string();

    if let Some(bin_name) = &args.bin {
        let has_bin = pkg
            .targets
            .iter()
            .any(|t| t.kind.iter().any(|k| k.to_string() == "bin") && t.name == *bin_name);
        if !has_bin {
            eprintln!(
                "error: no bin target named `{}` found in package `{}`",
                bin_name, package_name
            );
            return ExitCode::FAILURE;
        }
    } else {
        let bin_targets: Vec<_> = pkg
            .targets
            .iter()
            .filter(|t| t.kind.iter().any(|k| k.to_string() == "bin"))
            .collect();
        if bin_targets.is_empty() {
            eprintln!("error: a bin target must be available for `forge run`");
            return ExitCode::FAILURE;
        }
    }

    let common_args = CommonArgs {
        path: args.path,
        jobs: args.jobs,
        verbose: args.verbose,
        no_cache: false,
        sandbox: false,
        timeout_secs: None,
        profile: None,
        tui: false,
        remote_cache: None,
        remote_cache_token: None,
        remote_workers: None,
        remote_workers_token: None,
        cache_dir: None,
        send_source: false,
        ram_limit: None,
        semantic: false,
        ramdisk: false,
        swarm: false,
        reflink: false,
        hermetic_trace: false,
        swarm_compute: false,
    };

    let build_status = run_build_mode(common_args, BuildMode::Build);
    if build_status != ExitCode::SUCCESS {
        return build_status;
    }

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("run");
    cmd.arg("--package").arg(&package_name);
    if let Some(bin) = &args.bin {
        cmd.arg("--bin").arg(bin);
    }
    if !args.args.is_empty() {
        cmd.arg("--");
        cmd.args(args.args);
    }

    match cmd.status() {
        Ok(status) => {
            if let Some(code) = status.code() {
                ExitCode::from(code as u8)
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("error: failed to execute `cargo run`: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_graph(args: GraphArgs) -> ExitCode {
    let start_dir = match resolve_start_dir(args.path.as_deref()) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let project = match Project::discover(&start_dir) {
        Ok(Some(project)) => project,
        Ok(None) => {
            eprintln!(
                "error: no Cargo project found in `{}` or any parent directory",
                start_dir.display()
            );
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let graph = match project.build_graph() {
        Ok(graph) => graph,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    match args.format {
        GraphFormat::Tree => render::print_graph_tree(&project, &graph),
        GraphFormat::Json => render::print_graph_json(&project, &graph),
        GraphFormat::Dot => render::print_graph_dot(&project, &graph),
    }

    ExitCode::SUCCESS
}

fn mode_verb(mode: BuildMode) -> &'static str {
    match mode {
        BuildMode::Build => "Building",
        BuildMode::Check => "Checking",
        BuildMode::Test => "Testing",
    }
}

fn default_jobs() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2)
}

/// Opens the local fingerprint cache, honouring `--cache-dir` and
/// `--no-cache`.
fn open_cache(args: &CommonArgs) -> Option<LocalCache> {
    if args.no_cache {
        return None;
    }
    let result = match &args.cache_dir {
        Some(dir) => LocalCache::new(dir),
        None => LocalCache::default_location(),
    };
    match result {
        Ok(cache) => {
            if !args.tui {
                render::print_cache_location(cache.root());
            }
            Some(cache)
        }
        Err(error) => {
            if !args.tui {
                eprintln!("warning: fingerprint cache disabled: {error}");
            }
            None
        }
    }
}

/// Builds the scheduler, enabling RAM backpressure when `--ram-limit` is
/// given. The throttled worker floor is `jobs / 2` (minimum 1).
fn make_scheduler(workers: usize, args: &CommonArgs) -> Scheduler {
    let mut scheduler = Scheduler::new(workers);
    if let Some(limit) = args.ram_limit {
        let floor = (workers / 2).max(1);
        scheduler = scheduler.with_ram_backpressure(limit, floor);
    }
    scheduler
}

fn resolve_start_dir(path: Option<&Path>) -> std::result::Result<PathBuf, String> {
    let base = match path {
        Some(path) => {
            if path.is_file() {
                return Err(format!(
                    "`{}` is a file; expected a project directory",
                    path.display()
                ));
            }
            path.to_path_buf()
        }
        None => std::env::current_dir()
            .map_err(|error| format!("failed to determine the current directory: {error}"))?,
    };
    std::fs::canonicalize(&base)
        .map_err(|error| format!("cannot access `{}`: {error}", base.display()))
}

fn plain_path(path: &Path) -> std::path::PathBuf {
    let text = path.to_string_lossy();
    if cfg!(windows) {
        if let Some(stripped) = text.strip_prefix(r"\\?\") {
            return std::path::PathBuf::from(stripped);
        }
    }
    path.to_path_buf()
}

fn run_cc_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = plain_path(start_dir);
    let config_path = start_dir.join("forge.cc.json");
    let config = match CcProjectConfig::from_file(&config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to read `forge.cc.json`: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = match CcBackend::new(config.language) {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let build_dir = start_dir.join("build");
    let mut task_graph = match backend.create_tasks_from_config(&config, &start_dir, &build_dir) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    execute_task_graph(&mut task_graph, args)
}

fn run_go_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = plain_path(start_dir);
    let config_path = start_dir.join("forge.go.json");
    let config = if config_path.exists() {
        match GoProjectConfig::from_file(&config_path) {
            Ok(cfg) => cfg,
            Err(err) => {
                eprintln!("error: failed to read `forge.go.json`: {err}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        let name = start_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app")
            .to_string();
        GoProjectConfig {
            name,
            package_path: "./...".to_string(),
            tags: vec![],
            ldflags: None,
            gcflags: None,
            run_tests: true,
            output_binary: None,
        }
    };

    let backend = match GoBackend::new() {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let build_dir = start_dir.join("build");
    let mut task_graph = match backend.create_tasks_from_config(&config, &start_dir, &build_dir) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    execute_task_graph(&mut task_graph, args)
}

fn run_ts_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = plain_path(start_dir);
    let config = match TsProjectConfig::discover_or_default(&start_dir) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to discover TypeScript/JavaScript project: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = TsBackend::new();
    let mut task_graph = match backend.build_task_graph(&config, &start_dir) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    execute_task_graph(&mut task_graph, args)
}

fn run_py_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = plain_path(start_dir);
    let config = match PyProjectConfig::discover_or_default(&start_dir) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to discover Python project: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = PyBackend::new();
    let mut task_graph = match backend.build_task_graph(&config, &start_dir) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    execute_task_graph(&mut task_graph, args)
}

fn run_java_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = plain_path(start_dir);
    let config = match JavaProjectConfig::detect(&start_dir) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to discover Java/Kotlin project: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = match JavaBackend::new() {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: failed to initialize Java backend: {err}");
            return ExitCode::FAILURE;
        }
    };

    let build_dir = start_dir.join("build");
    let mut task_graph = match backend.create_tasks_from_config(&config, &start_dir, &build_dir) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    execute_task_graph(&mut task_graph, args)
}

fn run_dotnet_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = plain_path(start_dir);
    let config = match DotnetProjectConfig::detect(&start_dir) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to discover .NET project: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = match DotnetBackend::new() {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: failed to initialize .NET backend: {err}");
            return ExitCode::FAILURE;
        }
    };

    let build_dir = start_dir.join("build");
    let mut task_graph = match backend.create_tasks_from_config(&config, &start_dir, &build_dir) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    execute_task_graph(&mut task_graph, args)
}

fn run_swift_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = plain_path(start_dir);
    let config = match SwiftProjectConfig::detect(&start_dir) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to discover Swift/Objective-C project: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = match SwiftBackend::new() {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: failed to initialize Swift backend: {err}");
            return ExitCode::FAILURE;
        }
    };

    let build_dir = start_dir.join("build");
    let mut task_graph = match backend.create_tasks_from_config(&config, &start_dir, &build_dir) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    execute_task_graph(&mut task_graph, args)
}

fn run_dart_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = plain_path(start_dir);
    let config = match DartProjectConfig::detect(&start_dir) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to discover Dart/Flutter project: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = match DartBackend::new() {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: failed to initialize Dart backend: {err}");
            return ExitCode::FAILURE;
        }
    };

    let build_dir = start_dir.join("build");
    let mut task_graph = match backend.create_tasks_from_config(&config, &start_dir, &build_dir) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    execute_task_graph(&mut task_graph, args)
}

fn run_zig_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = plain_path(start_dir);
    let config = match ZigProjectConfig::detect(&start_dir) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to discover Zig project: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = match ZigBackend::new() {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: failed to initialize Zig backend: {err}");
            return ExitCode::FAILURE;
        }
    };

    let build_dir = start_dir.join("build");
    let mut task_graph = match backend.create_tasks_from_config(&config, &start_dir, &build_dir) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    execute_task_graph(&mut task_graph, args)
}

fn run_docker_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = plain_path(start_dir);
    
    let config = match DockerBackend::detect_config(&start_dir) {
        Some(cfg) => cfg,
        None => {
            eprintln!("error: failed to discover Docker project");
            return ExitCode::FAILURE;
        }
    };

    let backend = match DockerBackend::new(config) {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: failed to initialize Docker backend: {err}");
            return ExitCode::FAILURE;
        }
    };

    let mut task_graph = match backend.build_task_graph() {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    execute_task_graph(&mut task_graph, args)
}

fn run_plugin_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = plain_path(start_dir);
    let manifest = match PluginRulesManifest::discover_or_load(&start_dir) {
        Ok(m) => m,
        Err(err) => {
            eprintln!("error: failed to load custom build rules: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = PluginBackend::new();
    let mut task_graph = match backend.build_task_graph(&manifest, &start_dir) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    execute_task_graph(&mut task_graph, args)
}

fn execute_task_graph(
    task_graph: &mut forge_graph::BuildGraph<Task>,
    args: &CommonArgs,
) -> ExitCode {
    let cache = open_cache(args);
    let cache_handle = cache.clone();
    let executor = build_executor(args, cache);

    let workers = args.jobs.unwrap_or_else(default_jobs).max(1);
    let scheduler = make_scheduler(workers, args);

    let summary = if args.tui {
        let mut dashboard = TuiDashboard::new(task_graph.len());
        let _ = dashboard.start();
        let run_res = scheduler.run(task_graph, &executor, |task, outcome| {
            dashboard.on_task_finish(&task.label, outcome);
        });
        let summary = match run_res {
            Ok(s) => s,
            Err(err) => {
                let _ = dashboard.finish(&forge_scheduler::BuildSummary {
                    total: task_graph.len(),
                    executed: 0,
                    cached: 0,
                    failed: 1,
                    cancelled: 0,
                    duration: std::time::Duration::from_millis(0),
                    workers,
                    failures: vec![],
                    timings: vec![],
                });
                eprintln!("error: scheduling failure: {err}");
                return ExitCode::FAILURE;
            }
        };
        let _ = dashboard.finish(&summary);
        summary
    } else {
        match scheduler.run(task_graph, &executor, |task, outcome| {
            render::print_progress(task, outcome)
        }) {
            Ok(summary) => summary,
            Err(err) => {
                eprintln!("error: scheduling failure: {err}");
                return ExitCode::FAILURE;
            }
        }
    };

    render::print_failures(&summary);
    render::print_build_summary(&summary, BuildMode::Build);
    if let Some(ref c) = cache_handle {
        render::print_cache_stats(c);
    }
    if let Some(ref trace_path) = args.profile {
        if let Err(err) = summary.write_chrome_trace(trace_path) {
            eprintln!("warning: failed to write profile trace: {err}");
        } else {
            render::print_profile_saved(trace_path);
        }
    }
    if summary.succeeded() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

// Helper functions for file detection
fn has_file_with_extension(dir: &Path, extensions: &[&str]) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.iter().any(|e| *e == ext) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn has_dir_with_extension(dir: &Path, extensions: &[&str]) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(ext) = path.extension() {
                    if extensions.iter().any(|e| *e == ext) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
