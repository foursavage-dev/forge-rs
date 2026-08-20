use crate::utils::resolve_start_dir;
use fish_core::project::Project;
use std::path::Path;
use std::process::ExitCode;

pub fn run_clean(path: Option<&Path>) -> ExitCode {
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
