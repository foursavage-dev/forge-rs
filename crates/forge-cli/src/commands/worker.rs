use std::process::ExitCode;

use crate::args::WorkerArgs;
use forge_worker::WorkerServer;

pub fn run_worker(args: WorkerArgs) -> ExitCode {
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
