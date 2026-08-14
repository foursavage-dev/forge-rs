use std::fs;
use std::path::Path;

use forge_core::project::Project;
use tempfile::TempDir;

fn fixture(files: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().expect("create temp dir");
    for (relative, content) in files {
        let path = dir.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dirs");
        }
        fs::write(path, content).expect("write fixture file");
    }
    dir
}

const SIMPLE_MANIFEST: &str = r#"
[package]
name = "sample"
version = "0.1.0"
edition = "2024"
"#;

const WORKSPACE_MANIFEST: &str = r#"
[workspace]
resolver = "2"
members = ["network", "core", "app"]
"#;

fn package_manifest(name: &str, dependencies: &[(&str, &str)]) -> String {
    let mut manifest = format!(
        r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "2024"
"#
    );
    if !dependencies.is_empty() {
        manifest.push_str("\n[dependencies]\n");
        for (dep, path) in dependencies {
            manifest.push_str(&format!("{dep} = {{ path = \"{path}\" }}\n"));
        }
    }
    manifest
}

fn workspace_fixture() -> TempDir {
    fixture(&[
        ("Cargo.toml", WORKSPACE_MANIFEST),
        ("network/Cargo.toml", &package_manifest("network", &[])),
        ("network/src/lib.rs", "pub fn ping() {}\n"),
        (
            "core/Cargo.toml",
            &package_manifest("core", &[("network", "../network")]),
        ),
        ("core/src/lib.rs", "pub fn run() { network::ping(); }\n"),
        (
            "app/Cargo.toml",
            &package_manifest("app", &[("core", "../core")]),
        ),
        ("app/src/lib.rs", "pub fn start() { core::run(); }\n"),
    ])
}

fn member_names(project: &Project) -> Vec<String> {
    project
        .build_order()
        .iter()
        .map(|id| {
            project
                .package(id)
                .expect("member is present in metadata")
                .name
                .to_string()
        })
        .collect()
}

fn position(names: &[String], name: &str) -> usize {
    names
        .iter()
        .position(|n| n == name)
        .unwrap_or_else(|| panic!("expected `{name}` among {names:?}"))
}

#[test]
fn discovers_and_loads_single_package_project() {
    let dir = fixture(&[
        ("Cargo.toml", SIMPLE_MANIFEST),
        ("src/lib.rs", "pub fn answer() -> u32 { 42 }\n"),
    ]);

    let project = Project::discover(dir.path())
        .expect("load metadata")
        .expect("manifest should be found");

    assert_eq!(
        project.root_package().map(|p| p.name.to_string()),
        Some("sample".to_string())
    );
    assert!(!project.is_workspace());
    assert_eq!(project.workspace_members().len(), 1);
    assert_eq!(member_names(&project), vec!["sample".to_string()]);
}

#[test]
fn discovers_project_from_nested_directory() {
    let dir = fixture(&[
        ("Cargo.toml", SIMPLE_MANIFEST),
        ("src/lib.rs", "pub fn answer() -> u32 { 42 }\n"),
    ]);

    let project = Project::discover(&dir.path().join("src"))
        .expect("load metadata")
        .expect("manifest should be found");

    assert_eq!(
        project.root_package().map(|p| p.name.to_string()),
        Some("sample".to_string())
    );
    assert_eq!(
        project.manifest_path(),
        dir.path().join(Path::new("Cargo.toml")).as_path()
    );
}

#[test]
fn loads_from_explicit_manifest_path() {
    let dir = fixture(&[("Cargo.toml", SIMPLE_MANIFEST), ("src/lib.rs", "")]);

    let project = Project::load(&dir.path().join("Cargo.toml")).expect("load metadata");

    assert_eq!(
        project.root_package().map(|p| p.name.to_string()),
        Some("sample".to_string())
    );
    assert_eq!(
        project.workspace_root(),
        dir.path().as_os_str().to_str().unwrap()
    );
}

#[test]
fn returns_none_outside_a_cargo_project() {
    let dir = tempfile::tempdir().expect("create temp dir");
    assert!(
        Project::discover(dir.path())
            .expect("discovery has no metadata dependency")
            .is_none()
    );
}

#[test]
fn reports_error_for_nonexistent_manifest() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let missing = dir.path().join("Cargo.toml");
    let error = Project::load(&missing).expect_err("loading must fail");
    assert!(error.to_string().contains("failed to load Cargo metadata"));
}

#[test]
fn workspace_members_and_default_members_are_reported() {
    let dir = workspace_fixture();
    let project = Project::discover(dir.path())
        .expect("load metadata")
        .expect("manifest should be found");

    assert!(project.is_workspace());
    assert_eq!(project.workspace_members().len(), 3);
    assert_eq!(project.default_workspace_members().len(), 3);
}

#[test]
fn build_order_respects_dependencies() {
    let dir = workspace_fixture();
    let project = Project::discover(dir.path())
        .expect("load metadata")
        .expect("manifest should be found");

    let names = member_names(&project);
    assert_eq!(names.len(), 3);
    assert_eq!(
        names.iter().filter(|n| *n == "app").count(),
        1,
        "members must appear exactly once"
    );
    assert!(position(&names, "network") < position(&names, "core"));
    assert!(position(&names, "core") < position(&names, "app"));
}

#[test]
fn build_graph_mirrors_metadata_dependencies() {
    let dir = workspace_fixture();
    let project = Project::discover(dir.path())
        .expect("load metadata")
        .expect("manifest should be found");

    let graph = project.build_graph().expect("build graph");
    graph.validate().expect("graph must be valid");
    assert_eq!(graph.len(), 3);

    let name_by_id: Vec<&str> = graph
        .nodes()
        .iter()
        .map(|node| {
            project
                .package(&node.payload)
                .map(|package| package.name.as_str())
                .unwrap_or("?")
        })
        .collect();
    assert_eq!(name_by_id, vec!["network", "core", "app"]);

    let id_of = |name: &str| {
        graph
            .nodes()
            .iter()
            .find(|node| {
                project
                    .package(&node.payload)
                    .is_some_and(|package| package.name.as_str() == name)
            })
            .map(|node| node.id)
            .expect("node exists")
    };

    assert_eq!(graph.deps(id_of("app")), Ok(&[id_of("core")][..]));
    assert_eq!(graph.deps(id_of("core")), Ok(&[id_of("network")][..]));
    assert_eq!(graph.deps(id_of("network")), Ok(&[][..]));

    let levels = graph.levels();
    assert_eq!(
        levels,
        vec![
            vec![id_of("network")],
            vec![id_of("core")],
            vec![id_of("app")]
        ]
    );
    assert_eq!(graph.ready_nodes(), vec![id_of("network")]);
}

#[test]
fn build_graph_ignores_dev_only_edges() {
    let dir = fixture(&[
        ("Cargo.toml", WORKSPACE_MANIFEST),
        (
            "network/Cargo.toml",
            &dev_dep_manifest("network", "app", "../app"),
        ),
        ("network/src/lib.rs", "pub fn ping() {}\n"),
        (
            "core/Cargo.toml",
            &package_manifest("core", &[("network", "../network")]),
        ),
        ("core/src/lib.rs", "pub fn run() { network::ping(); }\n"),
        (
            "app/Cargo.toml",
            &package_manifest("app", &[("core", "../core")]),
        ),
        ("app/src/lib.rs", "pub fn start() { core::run(); }\n"),
    ]);

    let project = Project::discover(dir.path())
        .expect("load metadata")
        .expect("manifest should be found");
    let graph = project.build_graph().expect("build graph");
    graph.validate().expect("graph must remain acyclic");
    assert_eq!(graph.len(), 3);
    assert_eq!(graph.levels().len(), 3);
}

#[test]
fn build_test_graph_includes_acyclic_dev_edges() {
    let dir = fixture(&[
        ("Cargo.toml", WORKSPACE_MANIFEST),
        ("network/Cargo.toml", &package_manifest("network", &[])),
        ("network/src/lib.rs", "pub fn ping() {}\n"),
        ("core/Cargo.toml", &package_manifest("core", &[])),
        ("core/src/lib.rs", "pub fn run() {}\n"),
        (
            "app/Cargo.toml",
            &dev_dep_manifest("app", "core", "../core"),
        ),
        ("app/src/lib.rs", "pub fn start() {}\n"),
    ]);

    let project = Project::discover(dir.path())
        .expect("load metadata")
        .expect("manifest should be found");

    let build = project.build_graph().expect("build graph");
    build.validate().expect("build graph acyclic");
    assert_eq!(build.len(), 3, "build graph ignores dev edges");
    let level_of = |graph: &forge_graph::BuildGraph<cargo_metadata::PackageId>, name: &str| {
        graph
            .levels()
            .iter()
            .position(|level| {
                level.iter().any(|id| {
                    let package_id = &graph.node(*id).unwrap().payload;
                    project
                        .package(package_id)
                        .is_some_and(|p| p.name.as_str() == name)
                })
            })
            .unwrap()
    };
    assert_eq!(
        level_of(&build, "core"),
        level_of(&build, "app"),
        "no core->app edge in the build graph"
    );

    let test = project.build_test_graph().expect("test graph");
    test.validate().expect("test graph acyclic");
    assert_eq!(test.len(), 3, "test graph has the same members");
    assert!(
        level_of(&test, "core") < level_of(&test, "app"),
        "test graph schedules core before app via the dev edge"
    );
}

#[test]
fn build_test_graph_skips_cyclic_dev_edges() {
    let dir = fixture(&[
        ("Cargo.toml", WORKSPACE_MANIFEST),
        (
            "network/Cargo.toml",
            &dev_dep_manifest("network", "app", "../app"),
        ),
        ("network/src/lib.rs", "pub fn ping() {}\n"),
        (
            "core/Cargo.toml",
            &package_manifest("core", &[("network", "../network")]),
        ),
        ("core/src/lib.rs", "pub fn run() { network::ping(); }\n"),
        (
            "app/Cargo.toml",
            &package_manifest("app", &[("core", "../core")]),
        ),
        ("app/src/lib.rs", "pub fn start() { core::run(); }\n"),
    ]);

    let project = Project::discover(dir.path())
        .expect("load metadata")
        .expect("manifest should be found");
    let test = project.build_test_graph().expect("test graph");
    test.validate().expect("test graph stays acyclic");
    assert_eq!(test.len(), 3);
}

fn dev_dep_manifest(name: &str, dev_dep: &str, path: &str) -> String {
    format!(
        r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dev-dependencies]
{dev_dep} = {{ path = "{path}" }}
"#
    )
}
