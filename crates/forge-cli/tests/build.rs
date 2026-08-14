use std::fs;
use std::process::{Command, Output};

use tempfile::TempDir;

fn forge() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

fn run(command: &mut Command) -> Output {
    command.output().expect("failed to spawn forge")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

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

const WORKSPACE_MANIFEST: &str = r#"
[workspace]
resolver = "2"
members = ["network", "core", "app"]
"#;

const SIMPLE_MANIFEST: &str = r#"
[package]
name = "sample"
version = "0.1.0"
edition = "2024"
"#;

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

#[test]
fn version_flag_prints_name_and_version() {
    let output = run(forge().arg("--version"));
    assert!(output.status.success());
    assert!(stdout(&output).contains("forge 0.1.0"));
}

#[test]
fn version_subcommand_prints_name_and_version() {
    let output = run(forge().arg("version"));
    assert!(output.status.success());
    assert!(stdout(&output).contains("forge 0.1.0"));
}

#[test]
fn help_lists_commands() {
    let output = run(forge().arg("--help"));
    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("build"));
    assert!(text.contains("check"));
    assert!(text.contains("clean"));
    assert!(text.contains("version"));
}

#[test]
fn build_builds_workspace_successfully() {
    let dir = workspace_fixture();
    let output = run(forge().arg("build").current_dir(dir.path()));

    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let text = stdout(&output);
    assert!(text.contains("Forge"));
    assert!(text.contains("Project:"));
    assert!(text.contains("Workspace:"));
    assert!(text.contains("Build graph:"));
    assert!(text.contains("Building..."));
    assert!(text.contains("✓ network"));
    assert!(text.contains("✓ core"));
    assert!(text.contains("✓ app"));
    assert!(text.contains("Build completed successfully."));
    assert!(text.contains("Cache:"));

    let network = text.find("network").expect("network in summary");
    let core = text.find("core").expect("core in summary");
    let app = text.find("app").expect("app in summary");
    assert!(network < core, "network must be printed before core");
    assert!(core < app, "core must be printed before app");
    assert!(stderr(&output).is_empty(), "no errors on success");
}

#[test]
fn build_rebuilds_from_the_fingerprint_cache() {
    let dir = workspace_fixture();
    let first = run(forge().arg("build").current_dir(dir.path()));
    assert!(first.status.success(), "first build must succeed");

    let second = run(forge().arg("build").current_dir(dir.path()));
    assert!(second.status.success(), "rebuild must succeed");
    let text = stdout(&second);
    assert!(text.contains("✓ network (cached)"));
    assert!(text.contains("✓ core (cached)"));
    assert!(text.contains("✓ app (cached)"));
    assert!(text.contains("3 hits"), "cache hits reported: {text}");
}

#[test]
fn build_discovers_project_upward_from_nested_directory() {
    let dir = fixture(&[
        ("Cargo.toml", SIMPLE_MANIFEST),
        ("src/lib.rs", "pub fn answer() -> u32 { 42 }\n"),
    ]);

    let output = run(forge().arg("build").current_dir(dir.path().join("src")));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("sample"));
    assert!(stdout(&output).contains("Project:"));
}

#[test]
fn build_reports_a_compilation_failure() {
    let dir = fixture(&[
        ("Cargo.toml", SIMPLE_MANIFEST),
        ("src/lib.rs", "pub fn broken() -> u32 { \"nope\" }\n"),
    ]);

    let output = run(forge().arg("build").current_dir(dir.path()));
    assert!(!output.status.success());
    assert!(stdout(&output).contains("Build failed."));
    let errors = stderr(&output);
    assert!(errors.contains("Task:      sample"));
    assert!(errors.contains("error"), "compiler output shown: {errors}");
}

#[test]
fn build_fails_in_non_cargo_directory() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let output = run(forge().arg("build").current_dir(dir.path()));

    assert!(!output.status.success());
    assert!(
        stderr(&output)
            .contains("no Cargo, C/C++, Go, TypeScript, Python, or Custom Rules project found")
    );
    assert!(stderr(&output).contains("hint:"));
}

#[test]
fn check_type_checks_the_workspace() {
    let dir = workspace_fixture();
    let output = run(forge().arg("check").current_dir(dir.path()));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("Check completed successfully."));
}

#[test]
fn test_runs_workspace_tests_and_reports_success() {
    let dir = workspace_fixture();

    let lib = dir.path().join("core/src/lib.rs");
    let mut content = fs::read_to_string(&lib).expect("read lib.rs");
    content.push_str("\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn passes() {\n        assert!(true);\n    }\n}\n");
    fs::write(&lib, content).expect("write lib.rs");

    let output = run(forge().arg("test").current_dir(dir.path()));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("Testing..."));
    assert!(text.contains("All tests passed."));
    assert!(text.contains("✓ core"));
}

#[test]
fn test_reports_a_failing_test() {
    let dir = workspace_fixture();
    let lib = dir.path().join("app/src/lib.rs");
    let content = format!(
        "{}\n#[cfg(test)]\nmod tests {{\n    #[test]\n    fn fails() {{\n        assert_eq!(1, 2);\n    }}\n}}\n",
        fs::read_to_string(&lib).expect("read lib.rs")
    );
    fs::write(&lib, content).expect("write lib.rs");

    let output = run(forge().arg("test").current_dir(dir.path()));
    assert!(!output.status.success());
    assert!(stdout(&output).contains("Some tests failed."));
    let errors = stderr(&output);
    assert!(errors.contains("Task:      app"));
    assert!(
        errors.contains("test result: FAILED"),
        "test output in failure report: {errors}"
    );
}

#[test]
fn clean_removes_build_artifacts() {
    let dir = workspace_fixture();
    let build = run(forge().arg("build").current_dir(dir.path()));
    assert!(build.status.success(), "build must succeed first");
    assert!(
        dir.path().join("target").exists(),
        "cargo target dir exists"
    );

    let output = run(forge().arg("clean").current_dir(dir.path()));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("Cleaned."));
    assert!(
        !dir.path().join("target").exists(),
        "clean removes the target directory"
    );
}

#[test]
fn build_rejects_a_file_path() {
    let dir = fixture(&[
        ("Cargo.toml", SIMPLE_MANIFEST),
        ("src/lib.rs", "pub fn answer() -> u32 { 42 }\n"),
    ]);
    let output = run(forge().arg("build").arg(dir.path().join("src/lib.rs")));

    assert!(!output.status.success());
    assert!(stderr(&output).contains("is a file; expected a project directory"));
}

#[test]
fn graph_tree_prints_workspace_graph() {
    let dir = workspace_fixture();
    let output = run(forge().arg("graph").current_dir(dir.path()));
    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let text = stdout(&output);
    assert!(text.contains("app"));
    assert!(text.contains("└── core"));
    assert!(text.contains("    └── network"));
}

#[test]
fn forge_toml_sets_worker_count_and_disable_flag_wins() {
    let dir = workspace_fixture();
    fs::write(
        dir.path().join("forge.toml"),
        "backend = \"rust\"\njobs = 1\nno_cache = true\n",
    )
    .expect("write forge.toml");

    let output = run(forge().arg("build").current_dir(dir.path()));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(
        text.contains("Workers:   1"),
        "forge.toml jobs=1 is applied: {text}"
    );
    assert!(
        text.contains("Cached:    0"),
        "forge.toml no_cache=true disables the cache: {text}"
    );

    let second = run(forge().arg("build").current_dir(dir.path()));
    assert!(second.status.success());
    assert!(stdout(&second).contains("Cached:    0"));
}

#[test]
fn invalid_forge_toml_is_a_clear_error() {
    let dir = workspace_fixture();
    fs::write(dir.path().join("forge.toml"), "not = [valid toml").expect("write forge.toml");

    let output = run(forge().arg("build").current_dir(dir.path()));
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("invalid `"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn build_with_sandbox_flag_succeeds() {
    let dir = workspace_fixture();
    let output = run(forge()
        .arg("build")
        .arg("--sandbox")
        .current_dir(dir.path()));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("✓ network"));
    assert!(text.contains("✓ core"));
    assert!(text.contains("✓ app"));
    assert!(text.contains("Executed:  3"));
}

#[test]
fn build_with_profile_flag_generates_trace_json() {
    let dir = workspace_fixture();
    let trace_path = dir.path().join("my_trace.json");
    let output = run(forge()
        .arg("build")
        .arg("--profile")
        .arg(&trace_path)
        .current_dir(dir.path()));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("Trace:"));
    assert!(trace_path.exists());

    let content = fs::read_to_string(&trace_path).expect("read trace.json");
    let json: serde_json::Value = serde_json::from_str(&content).expect("parse trace.json");
    assert!(json["traceEvents"].is_array());
    let events = json["traceEvents"].as_array().unwrap();
    assert!(events.iter().any(|e| e["name"] == "network"));
    assert!(events.iter().any(|e| e["name"] == "core"));
    assert!(events.iter().any(|e| e["name"] == "app"));
}

#[test]
fn watch_mode_runs_initial_build_and_watches() {
    let dir = workspace_fixture();
    let output = run(forge().arg("watch").arg("--once").current_dir(dir.path()));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("Watching for file changes"));
    assert!(text.contains("✓ network"));
    assert!(text.contains("✓ core"));
    assert!(text.contains("✓ app"));
    assert!(text.contains("Executed:  3"));
}

#[test]
fn build_typescript_project_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let pkg_json = r#"{
        "name": "frontend",
        "scripts": {
            "typecheck": "node -e \"process.exit(0)\"",
            "build": "node -e \"process.exit(0)\""
        }
    }"#;
    fs::write(dir.path().join("package.json"), pkg_json).expect("write package.json");
    fs::create_dir(dir.path().join("src")).expect("mkdir src");
    fs::write(
        dir.path().join("src").join("index.ts"),
        "console.log('forge ts');",
    )
    .expect("write index.ts");

    let output = run(forge().arg("build").current_dir(dir.path()));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("✓ frontend:typecheck"));
    assert!(text.contains("✓ frontend:build"));
    assert!(text.contains("Executed:  2"));
}

#[test]
fn build_python_project_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let forge_py = r#"{
        "name": "ai-service",
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
    fs::write(dir.path().join("forge.py.json"), forge_py).expect("write forge.py.json");
    fs::create_dir(dir.path().join("src")).expect("mkdir src");
    fs::write(
        dir.path().join("src").join("main.py"),
        "print('forge python')",
    )
    .expect("write main.py");

    let output = run(forge().arg("build").current_dir(dir.path()));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("✓ ai-service:lint"));
    assert!(text.contains("✓ ai-service:test"));
    assert!(text.contains("Executed:  2"));
}

#[test]
fn build_custom_rules_project_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let forgefile = r#"{
        "name": "custom-rule-app",
        "rules": [
            {
                "name": "gen",
                "command": "node",
                "args": ["-e", "process.exit(0)"],
                "depends_on": []
            },
            {
                "name": "package",
                "command": "node",
                "args": ["-e", "process.exit(0)"],
                "depends_on": ["gen"]
            }
        ]
    }"#;
    fs::write(dir.path().join("Forgefile.json"), forgefile).expect("write Forgefile.json");

    let output = run(forge().arg("build").current_dir(dir.path()));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("✓ custom-rule-app:gen"));
    assert!(text.contains("✓ custom-rule-app:package"));
    assert!(text.contains("Executed:  2"));
}

#[test]
fn build_with_tui_flag_succeeds() {
    let dir = workspace_fixture();
    let output = run(forge().arg("build").arg("--tui").current_dir(dir.path()));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
}
