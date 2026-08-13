//! Forge command-line interface.
//!
//! Milestone 3: `forge build` / `forge check` / `forge clean` drive the full
//! pipeline: Cargo metadata → package graph → task graph (fingerprinted) →
//! parallel scheduling → caching → summary. `forge clean` delegates to
//! `cargo clean`.

#![forbid(unsafe_code)]

mod render;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use forge_backend_rust::{BuildMode, RustBackend};
use forge_cache::{CachingExecutor, LocalCache};
use forge_core::project::Project;
use forge_executor::{ExecutorError, ProcessExecutor, Task, TaskExecutor, TaskOutcome};
use forge_scheduler::Scheduler;

#[derive(Debug, Parser)]
#[command(
    name = "forge",
    version,
    about = "🦀 Forge: a fast, cache-first build orchestration system for Rust and beyond.",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print version information.
    Version,
    /// Build the current Cargo project.
    Build(BuildArgs),
    /// Type-check the current Cargo project without producing artifacts.
    Check(CheckArgs),
    /// Build and run the tests of every workspace package.
    Test(TestArgs),
    /// Remove build artifacts (delegates to `cargo clean`).
    Clean(CleanArgs),
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

#[derive(Debug, Args)]
struct CommonArgs {
    /// Project directory; defaults to the current directory.
    path: Option<PathBuf>,
    /// Number of parallel worker processes.
    #[arg(short = 'j', long = "jobs")]
    jobs: Option<usize>,
    /// Print the output of task commands as they complete.
    #[arg(short = 'v', long)]
    verbose: bool,
    /// Skip the fingerprint cache (rebuild everything).
    #[arg(long = "no-cache")]
    no_cache: bool,
}

#[derive(Debug, Args)]
struct CleanArgs {
    /// Project directory; defaults to the current directory.
    path: Option<PathBuf>,
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
    }
}

/// The process executor, optionally wrapped in the fingerprint cache.
enum ExecutorChoice {
    Plain(ProcessExecutor),
    Cached(CachingExecutor<ProcessExecutor>),
}

impl TaskExecutor for ExecutorChoice {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        match self {
            Self::Plain(inner) => inner.execute(task),
            Self::Cached(inner) => inner.execute(task),
        }
    }
}

fn run_build_mode(args: CommonArgs, mode: BuildMode) -> ExitCode {
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
            eprintln!(
                "hint: run `forge build` from inside a Cargo project (a directory containing Cargo.toml)"
            );
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("hint: make sure `cargo` is installed and available on PATH");
            return ExitCode::FAILURE;
        }
    };

    let package_graph = match if mode == BuildMode::Test {
        project.build_test_graph()
    } else {
        project.build_graph()
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

    let cache = if args.no_cache {
        None
    } else {
        match LocalCache::default_location() {
            Ok(cache) => {
                render::print_cache_location(cache.root());
                Some(cache)
            }
            Err(error) => {
                eprintln!("warning: fingerprint cache disabled: {error}");
                None
            }
        }
    };
    let executor = match cache {
        Some(cache) => ExecutorChoice::Cached(CachingExecutor::new(
            ProcessExecutor::new(args.verbose),
            cache,
        )),
        None => ExecutorChoice::Plain(ProcessExecutor::new(args.verbose)),
    };

    let workers = args.jobs.unwrap_or_else(default_jobs).max(1);
    let scheduler = Scheduler::new(workers);

    render::print_project(&project, &package_graph);
    println!();
    println!("{}...", mode_verb(mode));
    println!();

    let summary = match scheduler.run(&mut task_graph, &executor, |task, outcome| {
        render::print_progress(task, outcome)
    }) {
        Ok(summary) => summary,
        Err(error) => {
            eprintln!("error: scheduler failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    render::print_build_summary(&summary, mode);
    if let ExecutorChoice::Cached(cached) = &executor {
        render::print_cache_stats(cached.cache());
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
