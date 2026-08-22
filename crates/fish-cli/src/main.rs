#![forbid(unsafe_code)]

mod ai_bridge;
mod args;
mod attestation;
mod backends;
mod build;
mod commands;
mod config;
mod critical_path;
pub mod daemon;
pub mod experimental;
mod monitoring;
pub mod pgo;
pub mod pipeline;
mod polyglot;
mod predictive;
mod ramdisk;
mod render;
mod semantic;
mod swarm;
mod timemachine;
mod tui;
mod utils;
mod watch;

use std::process::ExitCode;

use clap::Parser;

// Re-export argument types for use in main.rs
pub use args::{Cli, Command, CommonArgs};

use fish_backend_rust::BuildMode;
use fish_remote_cache::RemoteCacheServer;

// Import utility functions from utils module
use utils::resolve_start_dir;

// Import all argument types from args module
use args::CacheServerArgs;

fn init_logging() {
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("fish=info,warn"));

    // Enable profiling if RUST_PROFILE environment variable is set
    if std::env::var("RUST_PROFILE").is_ok() {
        let (chrome_layer, _guard) = tracing_chrome::ChromeLayerBuilder::new()
            .include_args(true)
            .build();

        tracing_subscriber::registry()
            .with(filter)
            .with(chrome_layer)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_line_number(true),
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_line_number(true),
            )
            .init();
    }
}

fn main() -> ExitCode {
    init_logging();

    let cli = Cli::parse();

    // Enable experimental features if flag is set
    if cli.experimental {
        experimental::enable();
    }

    match cli.command {
        Command::Version => {
            println!("fish {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Command::Ai(args) => commands::run_ai(args),
        Command::Why(args) => commands::run_why(args),
        Command::Lsp(_args) => commands::run_lsp(),
        Command::Init(args) => commands::run_init(args.path, args.force),
        Command::New(args) => commands::run_new(&args.name, args.template.as_deref(), args.path),
        Command::Build(args) => run_build_mode(args.common, BuildMode::Build),
        Command::Check(args) => run_build_mode(args.common, BuildMode::Check),
        Command::Test(args) => run_build_mode(args.common, BuildMode::Test),
        Command::Clean(args) => commands::run_clean(args.path.as_deref()),
        Command::Run(args) => commands::run_run(args),
        Command::Graph(args) => commands::run_graph(args),
        Command::CacheServer(args) => run_cache_server(args),
        Command::Worker(args) => commands::run_worker(args),
        Command::Affected(args) => commands::run_affected(args),
        Command::Doctor(args) => commands::run_doctor_with_ai(args.ai, args.fix),
        Command::Cache(args) => commands::run_cache(args),
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
        Command::Ci(args) => commands::run_ci(args),
        Command::History(args) => commands::run_history(args),
        Command::Rewind(args) => commands::run_rewind(args),
        Command::Attest(args) => commands::run_attest(args),
        Command::Verify(args) => commands::run_verify(args),
        Command::LivePatch(args) => commands::run_live_patch(args),
        Command::Jit(args) => commands::run_jit(args),
        Command::SuperOpt(args) => commands::run_super_opt(args),
        Command::Plugin(args) => commands::run_plugin(args),
        Command::Fix(args) => commands::run_fix(args),
        Command::Ui(args) => match commands::run_ui(args.port, args.open, args.path) {
            Ok(_) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("error: ui server failed: {err}");
                ExitCode::FAILURE
            }
        },
        Command::Query(args) => {
            let start_dir = match resolve_start_dir(args.path.as_deref()) {
                Ok(dir) => dir,
                Err(message) => {
                    eprintln!("error: {message}");
                    return ExitCode::FAILURE;
                }
            };
            match fish_core::project::Project::discover(&start_dir) {
                Ok(Some(project)) => match project.build_graph() {
                    Ok(graph) => {
                        let parsed = match fish_graph::parse_query(&args.expr) {
                            Ok(q) => q,
                            Err(err) => {
                                eprintln!("error: invalid query expression: {err}");
                                return ExitCode::FAILURE;
                            }
                        };
                        let engine = fish_graph::GraphQueryEngine::new(&graph, |pkg_id| {
                            project
                                .package(pkg_id)
                                .map(|p| p.name.to_string())
                                .unwrap_or_else(|| pkg_id.to_string())
                        });
                        let matches = engine.eval(&parsed);
                        for id in matches {
                            if let Some(node) = graph.node(id) {
                                let name = project
                                    .package(&node.payload)
                                    .map(|p| p.name.to_string())
                                    .unwrap_or_else(|| format!("{id:?}"));
                                println!("//{name}");
                            }
                        }
                        ExitCode::SUCCESS
                    }
                    Err(err) => {
                        eprintln!("error: failed to construct graph: {err}");
                        ExitCode::FAILURE
                    }
                },
                _ => {
                    eprintln!(
                        "error: no fish project discovered at {}",
                        start_dir.display()
                    );
                    ExitCode::FAILURE
                }
            }
        }
        Command::Daemon(args) => match args.command {
            args::DaemonCommand::Start { port } => {
                println!("🦀 Starting Fish Build Daemon on 127.0.0.1:{port}...");
                let daemon = daemon::FishDaemon::new(port);
                match daemon.start_in_background() {
                    Ok(_) => {
                        println!("Fish Daemon started successfully.");
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("error: failed to start daemon: {e}");
                        ExitCode::FAILURE
                    }
                }
            }
            args::DaemonCommand::Status { port } => {
                if daemon::FishDaemon::is_alive(port) {
                    let resp = daemon::FishDaemon::send_command(port, "STATUS")
                        .unwrap_or_else(|_| "CONNECTED".to_string());
                    println!("Daemon active on port {port}: {resp}");
                    ExitCode::SUCCESS
                } else {
                    println!("Daemon is not running on port {port}.");
                    ExitCode::FAILURE
                }
            }
            args::DaemonCommand::Stop { port } => {
                if daemon::FishDaemon::is_alive(port) {
                    let _ = daemon::FishDaemon::send_command(port, "SHUTDOWN");
                    println!("Daemon on port {port} stopped.");
                    ExitCode::SUCCESS
                } else {
                    println!("Daemon is not running on port {port}.");
                    ExitCode::SUCCESS
                }
            }
        },
    }
}

fn run_cache_server(args: CacheServerArgs) -> ExitCode {
    println!("🦀 Fish Remote Cache Server");
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

pub(crate) fn run_build_mode(args: CommonArgs, mode: BuildMode) -> ExitCode {
    build::run_build_mode_with(args, mode, None)
}
