//! End-to-end pipeline: metadata → task graph → scheduler → real cargo
//! builds → fingerprint cache. These tests compile the tiny fixture
//! workspace with real `cargo build` invocations.

use std::fs;
use std::path::Path;

use cargo_metadata::PackageId;
use forge_backend_rust::{BuildMode, RustBackend};
use forge_cache::{CachingExecutor, LocalCache};
use forge_core::project::Project;
use forge_executor::ProcessExecutor;
use forge_graph::BuildGraph;
use forge_scheduler::Scheduler;

/// A three-package workspace: `app -> core -> network`.
fn write_workspace(root: &Path) {
    let packages = [
        ("network", "\n"),
        ("core", "\npub fn from_network() {}\n"),
        ("app", "\npub fn from_core() {}\n"),
    ];
    for (name, src) in packages {
        let dir = root.join(name);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        )
        .unwrap();
        fs::write(dir.join("src/lib.rs"), format!("// {name}\n{src}\n")).unwrap();
    }
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"network\", \"core\", \"app\"]\n\n[workspace.package]\nedition = \"2021\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        root.join("core").join("Cargo.toml"),
        "[package]\nname = \"core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nnetwork = { path = \"../network\" }\n",
    )
    .unwrap();
    fs::write(
        root.join("app").join("Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ncore = { path = \"../core\" }\n",
    )
    .unwrap();
}

fn discover(root: &Path) -> (Project, BuildGraph<PackageId>) {
    let project = Project::discover(root)
        .expect("workspace discovered")
        .expect("workspace exists");
    let package_graph = project.build_graph().expect("package graph");
    (project, package_graph)
}

#[test]
fn create_tasks_builds_an_ordered_graph_with_cache_entries() {
    let dir = tempfile::tempdir().unwrap();
    write_workspace(dir.path());
    let (project, package_graph) = discover(dir.path());

    let backend = RustBackend::new().unwrap();
    let tasks = backend
        .create_tasks(&project, &package_graph, BuildMode::Build, true)
        .unwrap();

    let labels: Vec<&str> = tasks
        .nodes()
        .iter()
        .map(|node| node.payload.label.as_str())
        .collect();
    assert_eq!(labels, ["network", "core", "app"], "topological order");

    let app = &tasks.nodes()[2].payload;
    assert_eq!(
        tasks.deps(tasks.nodes()[2].id).unwrap(),
        &[tasks.nodes()[1].id]
    );
    assert!(app.spec.command_line().contains("build"));
    assert!(app.spec.command_line().contains("--package app"));
    assert!(app.cache.is_some(), "cache entries are attached by default");

    let keys: Vec<&str> = tasks
        .nodes()
        .iter()
        .map(|node| node.payload.cache.as_ref().unwrap().key.as_str())
        .collect();
    assert!(!keys.windows(2).any(|w| w[0] == w[1]), "keys must differ");
    assert!(
        keys[0].contains("/build/network"),
        "key embeds package + mode"
    );
}

#[test]
fn check_mode_uses_cargo_check() {
    let dir = tempfile::tempdir().unwrap();
    write_workspace(dir.path());
    let (project, package_graph) = discover(dir.path());
    let backend = RustBackend::new().unwrap();
    let tasks = backend
        .create_tasks(&project, &package_graph, BuildMode::Check, true)
        .unwrap();
    for node in tasks.nodes() {
        assert!(
            node.payload.spec.command_line().contains("check"),
            "{} should use cargo check",
            node.payload.label
        );
    }
}

/// Full pipeline: build, rebuild from cache, then invalidate transitively.
#[test]
fn pipeline_builds_incrementally_with_the_fingerprint_cache() {
    let dir = tempfile::tempdir().unwrap();
    write_workspace(dir.path());
    let (project, package_graph) = discover(dir.path());
    let backend = RustBackend::new().unwrap();
    let mut tasks = backend
        .create_tasks(&project, &package_graph, BuildMode::Build, true)
        .unwrap();
    let cache_root = dir.path().join(".test-cache");
    let cache = LocalCache::new(&cache_root).unwrap();
    let executor = CachingExecutor::new(ProcessExecutor::new(false), cache);
    let scheduler = Scheduler::new(2);

    let first = scheduler
        .run(&mut tasks, &executor, |_, _| {})
        .expect("cold build");
    assert_eq!(first.executed, 3, "cold cache: everything executes");
    assert_eq!(first.cached, 0);
    assert_eq!(first.failed, 0);

    let second = scheduler
        .run(&mut tasks, &executor, |_, _| {})
        .expect("warm rebuild");
    assert_eq!(second.cached, 3, "unchanged inputs hit the cache");
    assert_eq!(second.executed, 0);

    // Touch core only: core and app (its dependent) must rebuild, network stays cached.
    fs::write(
        dir.path().join("core/src/lib.rs"),
        "// core\npub fn from_network() {} // changed\n",
    )
    .unwrap();
    let mut tasks = backend
        .create_tasks(&project, &package_graph, BuildMode::Build, true)
        .unwrap();
    let third = scheduler
        .run(&mut tasks, &executor, |_, _| {})
        .expect("incremental rebuild");
    assert_eq!(third.cached, 1, "network unaffected");
    assert_eq!(third.executed, 2, "core and app rebuild");
    assert_eq!(third.failed, 0);

    // Touch network: the whole cone invalidates.
    fs::write(
        dir.path().join("network/src/lib.rs"),
        "// network\npub fn net() {} // changed\n",
    )
    .unwrap();
    let backend = RustBackend::new().unwrap();
    let mut tasks = backend
        .create_tasks(&project, &package_graph, BuildMode::Build, true)
        .unwrap();
    let fourth = scheduler
        .run(&mut tasks, &executor, |_, _| {})
        .expect("cone invalidation");
    assert_eq!(fourth.cached, 0);
    assert_eq!(fourth.executed, 3);
    assert_eq!(fourth.failed, 0);

    assert!(
        executor.cache().stats().hits() >= 4,
        "warm and partially-warm runs should produce hits"
    );
}

#[test]
fn pipeline_reports_compilation_failures() {
    let dir = tempfile::tempdir().unwrap();
    write_workspace(dir.path());
    fs::write(
        dir.path().join("app/src/lib.rs"),
        "// app\npub fn broken() -> u32 { \"not a number\" }\n",
    )
    .unwrap();
    let (project, package_graph) = discover(dir.path());
    let backend = RustBackend::new().unwrap();
    let mut tasks = backend
        .create_tasks(&project, &package_graph, BuildMode::Build, false)
        .unwrap();
    let executor = ProcessExecutor::new(false);
    let scheduler = Scheduler::new(2);
    let summary = scheduler.run(&mut tasks, &executor, |_, _| {}).unwrap();

    assert_eq!(summary.failed, 1, "app fails to compile");
    assert_eq!(summary.failures[0].label, "app");
    assert!(
        summary.failures[0].stderr.contains("error"),
        "stderr should contain the compiler diagnostic, got: {}",
        summary.failures[0].stderr
    );
}
