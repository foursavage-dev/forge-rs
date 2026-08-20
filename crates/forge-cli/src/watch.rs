use std::collections::HashSet;
use std::path::Path;
use std::process::ExitCode;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use clap::ValueEnum;
use notify::{Event, RecursiveMode, Watcher};

use crate::{CommonArgs, run_build_mode};
use forge_backend_rust::BuildMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum WatchAction {
    #[default]
    Build,
    Check,
    Test,
}

impl From<WatchAction> for BuildMode {
    fn from(action: WatchAction) -> Self {
        match action {
            WatchAction::Build => BuildMode::Build,
            WatchAction::Check => BuildMode::Check,
            WatchAction::Test => BuildMode::Test,
        }
    }
}

pub fn is_relevant_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    if path_str.contains("/target/")
        || path_str.contains("\\target\\")
        || path_str.contains("/build/")
        || path_str.contains("\\build\\")
        || path_str.contains("/.git/")
        || path_str.contains("\\.git\\")
        || path_str.contains("/.forge/")
        || path_str.contains("\\.forge\\")
    {
        return false;
    }

    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        if file_name.starts_with('.') || file_name.ends_with('~') || file_name.ends_with(".tmp") {
            return false;
        }

        if file_name == "Cargo.toml"
            || file_name == "Cargo.lock"
            || file_name == "forge.toml"
            || file_name == "forge.cc.json"
            || file_name == "forge.go.json"
            || file_name == "go.mod"
            || file_name == "go.sum"
        {
            return true;
        }
    }

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext,
            "rs" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "hxx" | "go" | "s" | "asm"
        )
    } else {
        false
    }
}

pub fn run_watch(
    common: CommonArgs,
    action: WatchAction,
    debounce_ms: u64,
    clear: bool,
    start_dir: &Path,
    once: bool,
    predictive: bool,
) -> ExitCode {
    let mode = BuildMode::from(action);
    let predictive_engine = crate::predictive::PredictiveEngine::new(predictive);

    if clear {
        print!("\x1B[2J\x1B[1;1H");
    }

    println!("Watching for file changes in {}...", start_dir.display());
    if predictive {
        println!("Predictive pre-compilation enabled ⚡");
    }
    let _ = run_build_mode(common.clone(), mode);

    if once {
        return ExitCode::SUCCESS;
    }

    let (tx, rx) = mpsc::channel();
    let mut watcher = match notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    }) {
        Ok(w) => w,
        Err(err) => {
            eprintln!("error: failed to initialize file watcher: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = watcher.watch(start_dir, RecursiveMode::Recursive) {
        eprintln!(
            "error: failed to watch directory `{}`: {err}",
            start_dir.display()
        );
        return ExitCode::FAILURE;
    }

    let debounce_dur = Duration::from_millis(debounce_ms.max(50));

    while let Ok(first_event) = rx.recv() {
        let mut changed_paths = HashSet::new();
        for p in first_event.paths {
            if is_relevant_path(&p) {
                predictive_engine.record_touch(&p, None);
                changed_paths.insert(p);
            }
        }

        let debounce_start = Instant::now();
        while debounce_start.elapsed() < debounce_dur {
            if let Ok(event) = rx.recv_timeout(Duration::from_millis(20)) {
                for p in event.paths {
                    if is_relevant_path(&p) {
                        predictive_engine.record_touch(&p, None);
                        changed_paths.insert(p);
                    }
                }
            }
        }

        if !changed_paths.is_empty() {
            if clear {
                print!("\x1B[2J\x1B[1;1H");
            }
            println!();
            if predictive {
                println!(
                    "Changes detected in {} files. Executing predictive build...",
                    changed_paths.len()
                );
            } else {
                println!(
                    "Changes detected in {} files. Rebuilding...",
                    changed_paths.len()
                );
            }
            println!();
            let _ = run_build_mode(common.clone(), mode);
            println!();
            println!("Watching for changes...");
        }
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_relevant_path_accepts_source_files() {
        assert!(is_relevant_path(Path::new("src/main.rs")));
        assert!(is_relevant_path(Path::new("src/lib.rs")));
        assert!(is_relevant_path(Path::new("src/server.go")));
        assert!(is_relevant_path(Path::new("include/header.h")));
        assert!(is_relevant_path(Path::new("src/native.cpp")));
        assert!(is_relevant_path(Path::new("Cargo.toml")));
        assert!(is_relevant_path(Path::new("forge.toml")));
        assert!(is_relevant_path(Path::new("forge.cc.json")));
    }

    #[test]
    fn test_is_relevant_path_rejects_ignored_files() {
        assert!(!is_relevant_path(Path::new("target/debug/app.exe")));
        assert!(!is_relevant_path(Path::new("build/output.o")));
        assert!(!is_relevant_path(Path::new(".git/HEAD")));
        assert!(!is_relevant_path(Path::new(".forge/cache/123.bin")));
        assert!(!is_relevant_path(Path::new("src/main.rs.tmp")));
        assert!(!is_relevant_path(Path::new("src/.#main.rs")));
    }
}
