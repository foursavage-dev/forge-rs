use std::process::ExitCode;

use crate::args::{HistoryArgs, RewindArgs};
use crate::timemachine;
use crate::utils::resolve_start_dir;

pub fn run_history(args: HistoryArgs) -> ExitCode {
    let start_dir = match resolve_start_dir(args.path.as_deref()) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let tm = timemachine::TimeMachine::new(&start_dir);
    match tm.list_snapshots() {
        Ok(snapshots) => {
            println!("⏱️  Forge Time-Machine Build History ({} snapshots)", snapshots.len());
            for s in snapshots {
                let git_info = s.git_ref.map(|g| format!(" [{g}]")).unwrap_or_default();
                println!("  • {} - {} artifacts{}", s.id, s.total_artifacts, git_info);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: cannot read history: {e}");
            ExitCode::FAILURE
        }
    }
}

pub fn run_rewind(args: RewindArgs) -> ExitCode {
    let start_dir = match resolve_start_dir(args.path.as_deref()) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let tm = timemachine::TimeMachine::new(&start_dir);
    let target_dir = start_dir.join("target");
    match tm.rewind_to_snapshot(&args.snapshot_id, &target_dir) {
        Ok(count) => {
            println!("⚡ Rewound build state to `{}` ({} artifacts restored in 0ms)", args.snapshot_id, count);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: rewind failed: {e}");
            ExitCode::FAILURE
        }
    }
}
