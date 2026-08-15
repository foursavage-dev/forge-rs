#![forbid(unsafe_code)]

//! Command-line argument definitions for forge CLI
//!
//! This module contains all argument structures and enums for CLI subcommands.

use std::path::PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};

use forge_backend_rust::BuildMode;
use crate::watch;

/// CLI entry point
#[derive(Debug, Parser)]
#[command(
    name = "forge",
    version = env!("CARGO_PKG_VERSION"),
    about = "🦀 Forge: a fast, cache-first build orchestration system for Rust and beyond.",
    long_about = None
)]
pub struct Cli {
    /// Enable experimental features (use at your own risk)
    #[arg(long, global = true)]
    pub experimental: bool,

    #[command(subcommand)]
    pub command: Command,
}

/// CLI subcommands
#[derive(Debug, Subcommand)]
pub enum Command {
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
    History(HistoryArgs),
    Rewind(RewindArgs),
    Attest(AttestArgs),
    Verify(VerifyArgs),
    LivePatch(LivePatchArgs),
    Jit(JitArgs),
    SuperOpt(SuperOptArgs),
    Plugin(PluginArgs),
}

/// Arguments for affected command
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

/// Affected build mode
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AffectedMode {
    Build,
    Check,
    Test,
}

impl AffectedMode {
    pub fn to_build_mode(self) -> BuildMode {
        match self {
            AffectedMode::Build => BuildMode::Build,
            AffectedMode::Check => BuildMode::Check,
            AffectedMode::Test => BuildMode::Test,
        }
    }
}

/// Arguments for cache command
#[derive(Debug, Args)]
pub struct CacheArgs {
    /// Cache directory; defaults to `~/.forge/cache`.
    #[arg(long)]
    pub dir: Option<PathBuf>,
    #[command(subcommand)]
    pub command: CacheCommand,
}

/// Cache subcommands
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

/// Arguments for CAS operations
#[derive(Debug, Args)]
pub struct CasArgs {
    #[command(subcommand)]
    pub command: CasCommand,
}

/// CAS subcommands
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

/// Arguments for CI command
#[derive(Debug, Args)]
pub struct CiArgs {
    #[command(subcommand)]
    pub command: CiCommand,
}

/// CI subcommands
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

/// Arguments for cache prune
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

/// Arguments for build command
#[derive(Debug, Args)]
pub struct BuildArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}

/// Arguments for check command
#[derive(Debug, Args)]
pub struct CheckArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}

/// Arguments for test command
#[derive(Debug, Args)]
pub struct TestArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}

/// Common arguments shared by build/check/test commands
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
    #[arg(long = "critical-path")]
    pub critical_path: bool,
    #[arg(long = "turbo-link")]
    pub turbo_link: bool,
    #[arg(long = "speculative")]
    pub speculative: bool,
    #[arg(long = "daemon-pool")]
    pub daemon_pool: bool,
    #[arg(long = "kernel-bypass")]
    pub kernel_bypass: bool,
    #[arg(long = "wasm-sandbox")]
    pub wasm_sandbox: bool,
    #[arg(long = "super-opt")]
    pub super_opt: bool,
}

/// Arguments for live-patch command
#[derive(Debug, Args)]
pub struct LivePatchArgs {
    pub process_id: u32,
    pub target_binary: PathBuf,
    pub path: Option<PathBuf>,
}

/// Arguments for JIT command
#[derive(Debug, Args)]
pub struct JitArgs {
    pub function_name: String,
    #[arg(default_value = "42")]
    pub value: i32,
}

/// Arguments for super-opt command
#[derive(Debug, Args)]
pub struct SuperOptArgs {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
}

/// Arguments for history command
#[derive(Debug, Args)]
pub struct HistoryArgs {
    pub path: Option<PathBuf>,
}

/// Arguments for rewind command
#[derive(Debug, Args)]
pub struct RewindArgs {
    pub snapshot_id: String,
    pub path: Option<PathBuf>,
}

/// Arguments for attestation command
#[derive(Debug, Args)]
pub struct AttestArgs {
    pub path: Option<PathBuf>,
}

/// Arguments for verify command
#[derive(Debug, Args)]
pub struct VerifyArgs {
    pub attestation_file: PathBuf,
    pub path: Option<PathBuf>,
}

/// Arguments for watch command
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

/// Arguments for clean command
#[derive(Debug, Args)]
pub struct CleanArgs {
    pub path: Option<PathBuf>,
}

/// Arguments for run command
#[derive(Debug, Args)]
pub struct RunArgs {
    pub path: Option<PathBuf>,
    #[arg(short = 'p', long)]
    pub package: Option<String>,
    #[arg(long)]
    pub bin: Option<String>,
    #[arg(short = 'j', long = "jobs")]
    pub jobs: Option<usize>,
    #[arg(short = 'v', long)]
    pub verbose: bool,
    #[arg(last = true)]
    pub args: Vec<String>,
}

/// Arguments for graph command
#[derive(Debug, Args)]
pub struct GraphArgs {
    pub path: Option<PathBuf>,
    #[arg(long, default_value_t = GraphFormat::Tree, value_enum)]
    pub format: GraphFormat,
}

/// Arguments for cache server
#[derive(Debug, Args)]
pub struct CacheServerArgs {
    #[arg(long, default_value = "127.0.0.1:9091")]
    pub listen: String,
    #[arg(long)]
    pub auth_token: Option<String>,
    #[arg(long)]
    pub dir: Option<PathBuf>,
}

/// Arguments for worker
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

/// Arguments for plugin command
#[derive(Debug, Args)]
pub struct PluginArgs {
    /// Project directory
    #[arg(long, short)]
    pub path: Option<PathBuf>,
    #[command(subcommand)]
    pub action: PluginAction,
}

/// Plugin subcommands
#[derive(Debug, Subcommand)]
pub enum PluginAction {
    /// List all available script plugins
    List,
    /// Execute a specific plugin command
    Execute {
        /// Plugin name
        name: String,
        /// Command to execute
        command: String,
        /// Additional arguments for the command
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// Graph output format
#[derive(Debug, Clone, clap::ValueEnum, Default)]
pub enum GraphFormat {
    #[default]
    Tree,
    Json,
    Dot,
}
