use std::fs;
use tempfile::tempdir;

use forge_backend_py::{PyBackend, PyProjectConfig};
use forge_executor::ProcessExecutor;
use forge_scheduler::Scheduler;

#[test]
fn py_pipeline_builds_custom_task_graph_and_runs() {
    let dir = tempdir().unwrap();
    let forge_json = r#"{
        "name": "my-ml-app",
        "tasks": [
            {
                "name": "lint",
                "command": "node",
                "args": ["-e", "process.exit(0)"],
                "depends_on": []
            },
            {
                "name": "test",
                "command": "node",
                "args": ["-e", "process.exit(0)"],
                "depends_on": ["lint"]
            }
        ]
    }"#;
    fs::write(dir.path().join("forge.py.json"), forge_json).unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src").join("app.py"), "def run(): pass").unwrap();

    let config = PyProjectConfig::discover_or_default(dir.path()).unwrap();
    let backend = PyBackend::new();
    let mut graph = backend.build_task_graph(&config, dir.path()).unwrap();

    assert_eq!(graph.len(), 2);

    let executor = ProcessExecutor::new(false);
    let scheduler = Scheduler::new(2);
    let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();

    assert_eq!(summary.total, 2);
    assert_eq!(summary.executed, 2);
    assert_eq!(summary.failed, 0);
}
