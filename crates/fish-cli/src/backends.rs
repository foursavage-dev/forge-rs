use std::path::Path;
use std::process::ExitCode;

use fish_backend_cc::{CcBackend, CcProjectConfig};
use fish_backend_dart::{DartBackend, DartProjectConfig};
use fish_backend_docker::DockerBackend;
use fish_backend_dotnet::{DotnetBackend, DotnetProjectConfig};
use fish_backend_go::{GoBackend, GoProjectConfig};
use fish_backend_java::{JavaBackend, JavaProjectConfig};
use fish_backend_py::{PyBackend, PyProjectConfig};
use fish_backend_rust::BuildMode;
use fish_backend_swift::{SwiftBackend, SwiftProjectConfig};
use fish_backend_ts::{TsBackend, TsProjectConfig};
use fish_backend_zig::{ZigBackend, ZigProjectConfig};

use fish_executor::Task;
use fish_plugin::{
    PluginBackend, PluginRulesManifest,
    scripting::{PluginError, PluginManager},
};

use crate::args::CommonArgs;
use crate::render;
use crate::tui::TuiDashboard;
use crate::utils;

pub(crate) fn run_cc_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = utils::plain_path(start_dir);
    let config_path = start_dir.join("fish.cc.json");
    let config = match CcProjectConfig::from_file(&config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to read `fish.cc.json`: {err}");
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

pub(crate) fn run_go_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = utils::plain_path(start_dir);
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

pub(crate) fn run_ts_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = utils::plain_path(start_dir);
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

pub(crate) fn run_py_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = utils::plain_path(start_dir);
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

pub(crate) fn run_java_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = utils::plain_path(start_dir);
    let config = match JavaProjectConfig::detect(&start_dir) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to discover Java/Kotlin project: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = match JavaBackend::new() {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: failed to initialize Java backend: {err}");
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

pub(crate) fn run_dotnet_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = utils::plain_path(start_dir);
    let config = match DotnetProjectConfig::detect(&start_dir) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to discover .NET project: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = match DotnetBackend::new() {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: failed to initialize .NET backend: {err}");
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

pub(crate) fn run_swift_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = utils::plain_path(start_dir);
    let config = match SwiftProjectConfig::detect(&start_dir) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to discover Swift/Objective-C project: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = match SwiftBackend::new() {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: failed to initialize Swift backend: {err}");
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

pub(crate) fn run_dart_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = utils::plain_path(start_dir);
    let config = match DartProjectConfig::detect(&start_dir) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to discover Dart/Flutter project: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = match DartBackend::new() {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: failed to initialize Dart backend: {err}");
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

pub(crate) fn run_zig_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = utils::plain_path(start_dir);
    let config = match ZigProjectConfig::detect(&start_dir) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("error: failed to discover Zig project: {err}");
            return ExitCode::FAILURE;
        }
    };

    let backend = match ZigBackend::new() {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: failed to initialize Zig backend: {err}");
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

pub(crate) fn run_docker_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = utils::plain_path(start_dir);

    let config = match DockerBackend::detect_config(&start_dir) {
        Some(cfg) => cfg,
        None => {
            eprintln!("error: failed to discover Docker project");
            return ExitCode::FAILURE;
        }
    };

    let backend = match DockerBackend::new(config) {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: failed to initialize Docker backend: {err}");
            return ExitCode::FAILURE;
        }
    };

    let mut task_graph = match backend.build_task_graph() {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    execute_task_graph(&mut task_graph, args)
}

pub(crate) fn run_plugin_build(start_dir: &Path, args: &CommonArgs) -> ExitCode {
    let start_dir = utils::plain_path(start_dir);
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

/// Detects and loads script plugins from a .fish/plugins directory
pub(crate) fn detect_and_load_plugins(start_dir: &Path) -> Result<PluginManager, PluginError> {
    let plugin_dir = start_dir.join(".fish").join("plugins");
    let mut manager = PluginManager::new(plugin_dir);
    manager.load_plugins()?;
    Ok(manager)
}

/// Executes a specific script plugin by name
pub(crate) fn execute_script_plugin(
    start_dir: &Path,
    plugin_name: &str,
    command: &str,
    args: &[String],
) -> Result<fish_plugin::scripting::PluginOutput, PluginError> {
    let manager = detect_and_load_plugins(start_dir)?;
    manager.execute_plugin(plugin_name, command, args)
}

/// Checks if a project has any script plugins loaded
pub(crate) fn has_script_plugins(start_dir: &Path) -> bool {
    let plugin_dir = start_dir.join(".fish").join("plugins");
    if !plugin_dir.exists() {
        return false;
    }

    match detect_and_load_plugins(start_dir) {
        Ok(manager) => !manager.list_plugins().is_empty(),
        Err(_) => false,
    }
}

/// Lists all available script plugins in the project
pub(crate) fn list_script_plugins(start_dir: &Path) -> Vec<String> {
    match detect_and_load_plugins(start_dir) {
        Ok(manager) => manager
            .list_plugins()
            .iter()
            .map(|p| p.name.clone())
            .collect(),
        Err(_) => vec![],
    }
}

pub(crate) fn execute_task_graph(
    task_graph: &mut fish_graph::BuildGraph<Task>,
    args: &CommonArgs,
) -> ExitCode {
    let cache = utils::open_cache(args);
    let cache_handle = cache.clone();
    let executor = crate::build::build_executor(args, cache);

    let workers = args.jobs.unwrap_or_else(utils::default_jobs).max(1);
    let scheduler = utils::make_scheduler(workers, args);

    let summary = if args.tui {
        let mut dashboard = TuiDashboard::new(task_graph.len());
        let _ = dashboard.start();
        let run_res = scheduler.run(task_graph, &executor, |task, outcome| {
            dashboard.on_task_finish(&task.label, outcome);
        });
        let summary = match run_res {
            Ok(s) => s,
            Err(err) => {
                let _ = dashboard.finish(&fish_scheduler::BuildSummary {
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
    if let Some(ref c) = cache_handle {
        render::print_cache_stats(c);
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
        ExitCode::FAILURE
    }
}

// Helper functions for file detection
pub(crate) fn has_file_with_extension(dir: &Path, extensions: &[&str]) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension()
                && extensions.iter().any(|e| *e == ext)
            {
                return true;
            }
        }
    }
    false
}

pub(crate) fn has_dir_with_extension(dir: &Path, extensions: &[&str]) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && let Some(ext) = path.extension()
                && extensions.iter().any(|e| *e == ext)
            {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_has_script_plugins_no_directory() {
        let dir = tempdir().unwrap();
        assert!(!has_script_plugins(dir.path()));
    }

    #[test]
    fn test_has_script_plugins_empty_directory() {
        let dir = tempdir().unwrap();
        let plugin_dir = dir.path().join(".fish").join("plugins");
        fs::create_dir_all(&plugin_dir).unwrap();
        assert!(!has_script_plugins(dir.path()));
    }

    #[test]
    fn test_list_script_plugins_empty() {
        let dir = tempdir().unwrap();
        let plugins = list_script_plugins(dir.path());
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_list_script_plugins_with_valid_plugin() {
        let dir = tempdir().unwrap();
        let plugin_dir = dir.path().join(".fish").join("plugins");
        fs::create_dir_all(&plugin_dir).unwrap();

        let test_plugin_dir = plugin_dir.join("test-plugin");
        fs::create_dir_all(&test_plugin_dir).unwrap();

        let plugin_config = r#"{
            "name": "test-plugin",
            "version": "1.0.0",
            "script_type": "Shell",
            "entry_point": "/bin/sh",
            "dependencies": [],
            "capabilities": {
                "can_build": true,
                "can_test": false,
                "can_clean": false,
                "can_graph": false,
                "supports_watch": false
            }
        }"#;

        fs::write(test_plugin_dir.join("plugin.json"), plugin_config).unwrap();

        let plugins = list_script_plugins(dir.path());
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0], "test-plugin");
    }

    #[test]
    fn test_detect_and_load_plugins_no_directory() {
        let dir = tempdir().unwrap();
        let result = detect_and_load_plugins(dir.path());
        // Should succeed even without directory
        assert!(result.is_ok());
        let manager = result.unwrap();
        assert!(manager.list_plugins().is_empty());
    }
}
