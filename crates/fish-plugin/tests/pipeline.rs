use std::fs;
use tempfile::tempdir;

use fish_executor::ProcessExecutor;
use fish_plugin::{PluginBackend, PluginRulesManifest};
use fish_scheduler::Scheduler;

#[test]
fn plugin_rules_execute_end_to_end_in_scheduler() {
    let dir = tempdir().unwrap();
    let forgefile = r#"{
        "name": "proto-pipeline",
        "rules": [
            {
                "name": "generate_proto",
                "command": "node",
                "args": ["-e", "process.exit(0)"],
                "depends_on": []
            },
            {
                "name": "build_proto_client",
                "command": "node",
                "args": ["-e", "process.exit(0)"],
                "depends_on": ["generate_proto"]
            }
        ]
    }"#;
    fs::write(dir.path().join("forge.rules.json"), forgefile).unwrap();

    let manifest = PluginRulesManifest::discover_or_load(dir.path()).unwrap();
    let backend = PluginBackend::new();
    let mut graph = backend.build_task_graph(&manifest, dir.path()).unwrap();

    assert_eq!(graph.len(), 2);

    let executor = ProcessExecutor::new(false);
    let scheduler = Scheduler::new(2);
    let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();

    assert_eq!(summary.total, 2);
    assert_eq!(summary.executed, 2);
    assert_eq!(summary.failed, 0);
}
