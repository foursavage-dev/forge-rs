#![forbid(unsafe_code)]

mod args;
mod attestation;
mod backends;
mod build;
mod commands;
mod config;
mod critical_path;
pub mod experimental;
mod monitoring;
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

use forge_backend_rust::BuildMode;
use forge_remote_cache::RemoteCacheServer;

// Import utility functions from utils module
use utils::resolve_start_dir;

// Import all argument types from args module
use args::CacheServerArgs;

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Enable experimental features if flag is set
    if cli.experimental {
        experimental::enable();
    }

    match cli.command {
        Command::Version => {
            println!("forge {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
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
        Command::Doctor(args) => commands::run_doctor_with_ai(args.ai),
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
        Command::Fix(args) => match commands::run_fix(args.path, args.apply, args.ai) {
            Ok(_) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("error: fix diagnostic failed: {err}");
                ExitCode::FAILURE
            }
        },
        Command::Ui(args) => match commands::run_ui(args.port, args.open, args.path) {
            Ok(_) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("error: ui server failed: {err}");
                ExitCode::FAILURE
            }
        },
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

pub(crate) fn run_build_mode(args: CommonArgs, mode: BuildMode) -> ExitCode {
    build::run_build_mode_with(args, mode, None)
}
