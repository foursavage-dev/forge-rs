#![forbid(unsafe_code)]

mod config;
mod render;
mod tui;
mod watch;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use forge_backend_cc::{CcBackend, CcProjectConfig};
use forge_backend_go::{GoBackend, GoProjectConfig};
use forge_backend_py::{PyBackend, PyProjectConfig};
use forge_backend_rust::{BuildMode, RustBackend};
use forge_backend_ts::{TsBackend, TsProjectConfig};
use forge_cache::{CachingExecutor, LocalCache};
use forge_core::project::Project;
use forge_executor::{ExecutorError, ProcessExecutor, Task, TaskExecutor, TaskOutcome};
use forge_plugin::{PluginBackend, PluginRulesManifest};
use forge_sandbox::{SandboxConfig, SandboxedExecutor};
use forge_scheduler::Scheduler;
use tui::TuiDashboard;

use crate::config::{BackendChoice, ForgeConfig};

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
    Version,
    Build(BuildArgs),
    Check(CheckArgs),
    Test(TestArgs),
    Clean(CleanArgs),
    Run(RunArgs),
    Graph(GraphArgs),
    Watch(WatchArgs),
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
            )
        }
    }
}

enum ExecutorChoice {
    Plain(ProcessExecutor),
    Cached(CachingExecutor<ProcessExecutor>),
    SandboxedPlain(SandboxedExecutor<ProcessExecutor>),
    SandboxedCached(SandboxedExecutor<CachingExecutor<ProcessExecutor>>),
}

impl TaskExecutor for ExecutorChoice {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        match self {
            Self::Plain(inner) => inner.execute(task),
            Self::Cached(inner) => inner.execute(task),
            Self::SandboxedPlain(inner) => inner.execute(task),
            Self::SandboxedCached(inner) => inner.execute(task),
        }
    }
}

fn build_executor(args: &CommonArgs, cache: Option<LocalCache>) -> ExecutorChoice {
    let process = ProcessExecutor::with_timeout(
        args.verbose,
        args.timeout_secs.map(std::time::Duration::from_secs),
    );

    if args.sandbox {
        let sb_config = SandboxConfig::default();
        match cache {
            Some(c) => ExecutorChoice::SandboxedCached(SandboxedExecutor::new(
                CachingExecutor::new(process, c),
                sb_config,
            )),
            None => ExecutorChoice::SandboxedPlain(SandboxedExecutor::new(process, sb_config)),
        }
    } else {
        match cache {
            Some(c) => ExecutorChoice::Cached(CachingExecutor::new(process, c)),
            None => ExecutorChoice::Plain(process),
        }
    }
}

pub(crate) fn run_build_mode(args: CommonArgs, mode: BuildMode) -> ExitCode {
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
    };

    match config.backend {
        BackendChoice::Cc => return run_cc_build(&start_dir, &merged),
        BackendChoice::Go => return run_go_build(&start_dir, &merged),
        BackendChoice::Ts
        | BackendChoice::Typescript
        | BackendChoice::Javascript
        | BackendChoice::Js => return run_ts_build(&start_dir, &merged),
        BackendChoice::Py | BackendChoice::Python => return run_py_build(&start_dir, &merged),
        BackendChoice::Plugin | BackendChoice::Rules => {
            return run_plugin_build(&start_dir, &merged);
        }
        BackendChoice::Rust => return run_rust_build(&start_dir, &merged, mode),
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

    run_rust_build(&start_dir, &merged, mode)
}

fn run_rust_build(start_dir: &Path, args: &CommonArgs, mode: BuildMode) -> ExitCode {
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
    };
    let executor = build_executor(args, cache);

    let workers = args.jobs.unwrap_or_else(default_jobs).max(1);
    let scheduler = Scheduler::new(workers);

    if !args.tui {
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
    match &executor {
        ExecutorChoice::Cached(cached) => render::print_cache_stats(cached.cache()),
        ExecutorChoice::SandboxedCached(cached) => {
            render::print_cache_stats(cached.inner().cache())
        }
        _ => {}
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
    let cache = if args.no_cache {
        None
    } else {
        match LocalCache::default_location() {
            Ok(c) => {
                if !args.tui {
                    render::print_cache_location(c.root());
                }
                Some(c)
            }
            Err(err) => {
                if !args.tui {
                    eprintln!("warning: fingerprint cache disabled: {err}");
                }
                None
            }
        }
    };

    let executor = build_executor(args, cache);

    let workers = args.jobs.unwrap_or_else(default_jobs).max(1);
    let scheduler = Scheduler::new(workers);

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
