use std::fs;
use tempfile::tempdir;

use fish_backend_ts::{TsBackend, TsProjectConfig};
use fish_executor::ProcessExecutor;
use fish_scheduler::Scheduler;

#[test]
fn ts_pipeline_builds_task_graph_and_runs_scheduler() {
    let dir = tempdir().unwrap();
    let pkg_json = r#"{
        "name": "my-ts-app",
        "scripts": {
            "typecheck": "node -e \"process.exit(0)\"",
            "build": "node -e \"process.exit(0)\"",
            "test": "node -e \"process.exit(0)\""
        }
    }"#;
    fs::write(dir.path().join("package.json"), pkg_json).unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("src").join("main.ts"),
        "export const app = 42;",
    )
    .unwrap();

    let config = TsProjectConfig::discover_or_default(dir.path()).unwrap();
    let backend = TsBackend::new();
    let mut graph = backend.build_task_graph(&config, dir.path()).unwrap();

    assert_eq!(graph.len(), 3);

    let executor = ProcessExecutor::new(false);
    let scheduler = Scheduler::new(2);
    let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();

    assert_eq!(summary.total, 3);
    assert_eq!(summary.executed, 3);
    assert_eq!(summary.failed, 0);
}
