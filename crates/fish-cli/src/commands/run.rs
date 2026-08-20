use std::process::ExitCode;

use fish_backend_rust::BuildMode;
use fish_core::project::Project;

use crate::args::{CommonArgs, RunArgs};
use crate::utils::resolve_start_dir;

pub fn run_run(args: RunArgs) -> ExitCode {
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
            if let Some(pkg) = project.package(id)
                && pkg.name.as_str() == *pkg_name
            {
                target_pkg = Some(pkg);
                break;
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
        critical_path: false,
        turbo_link: false,
        speculative: false,
        daemon_pool: false,
        kernel_bypass: false,
        wasm_sandbox: false,
        super_opt: false,
        explain: false,
    };

    let build_status = crate::run_build_mode(common_args, BuildMode::Build);
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
