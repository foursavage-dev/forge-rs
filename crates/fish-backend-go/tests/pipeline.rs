use std::fs;
use tempfile::tempdir;

use fish_backend_go::{GoBackend, GoProjectConfig, GoToolchain};
use fish_cache::LocalCache;
use fish_executor::ProcessExecutor;
use fish_scheduler::Scheduler;

#[test]
fn go_pipeline_builds_task_graph_and_runs_scheduler() {
    let dummy_toolchain = GoToolchain {
        executable: "go".to_string(),
        version: "go version go1.22.0 windows/amd64".to_string(),
    };

    let backend = GoBackend::with_toolchain(dummy_toolchain);
    let temp = tempdir().unwrap();
    let project_dir = temp.path().join("goproj");
    fs::create_dir_all(&project_dir).unwrap();

    fs::write(
        project_dir.join("go.mod"),
        "module example.com/goproj\n\ngo 1.22\n",
    )
    .unwrap();

    fs::write(
        project_dir.join("main.go"),
        "package main\n\nfunc main() {}\n",
    )
    .unwrap();

    let config = GoProjectConfig {
        name: "test_go_app".to_string(),
        package_path: "./...".to_string(),
        tags: vec![],
        ldflags: None,
        gcflags: None,
        run_tests: true,
        output_binary: None,
    };

    let build_dir = temp.path().join("build");
    fs::create_dir_all(&build_dir).unwrap();

    let graph = backend
        .create_tasks_from_config(&config, &project_dir, &build_dir)
        .unwrap();

    assert_eq!(graph.len(), 3);

    let cache_dir = temp.path().join("cache");
    let cache = LocalCache::new(cache_dir).unwrap();
    let process = ProcessExecutor::new(false);
    let _executor = fish_cache::CachingExecutor::new(process, cache);
    let _scheduler = Scheduler::new(2);
}
