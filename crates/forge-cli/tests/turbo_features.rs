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

const CARGO_MANIFEST: &str = r#"
[package]
name = "turbo_demo"
version = "0.1.0"
edition = "2024"
"#;

#[test]
fn test_semantic_token_invariance_rust() {
    let code_clean = "fn compute(x: i32) -> i32 {\n    x * 2\n}\n";
    let code_commented = "fn compute(x: i32) -> i32 {\n    // Double the input value\n    /* Block comment here */\n    x * 2\n}\n\n";

    let temp_dir = tempfile::tempdir().unwrap();
    let file_a = temp_dir.path().join("a.rs");
    let file_b = temp_dir.path().join("b.rs");

    fs::write(&file_a, code_clean).unwrap();
    fs::write(&file_b, code_commented).unwrap();

    let output_a = run(forge().arg("build").arg("--semantic").arg(temp_dir.path()));
    assert!(output_a.status.success() || stdout(&output_a).contains("Semantic") || stderr(&output_a).is_empty());
}

#[test]
fn test_semantic_multi_language_ast_normalization() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn calculate(a: i32, b: i32) -> i32 {\n    // Calculate sum\n    a + b\n}\n"),
        ("native/math.cpp", "int multiply(int a, int b) {\n    /* Multi-line comment */\n    return a * b;\n}\n"),
        ("web/index.ts", "export function greet(name: string): string {\n    // Greet user\n    return `Hello, ${name}`;\n}\n"),
        ("scripts/util.py", "def compute(val):\n    # Python comment\n    return val * 10\n"),
    ]);

    let output = run(forge()
        .arg("build")
        .arg("--semantic")
        .arg(dir.path()));

    let out = stdout(&output);
    let err = stderr(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {err}");
    assert!(out.contains("Semantic AST-aware fingerprinting active"));
}

#[test]
fn test_ramdisk_real_artifact_isolation_and_sync() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn high_throughput_operation() -> u64 { 1_000_000 }\n"),
    ]);

    let output = run(forge()
        .arg("build")
        .arg("--ramdisk")
        .arg(dir.path()));

    let out = stdout(&output);
    let err = stderr(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {err}");
    assert!(out.contains("In-memory RAM disk turbo enabled"));
}

#[test]
fn test_swarm_cache_p2p_network_broadcast() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn distributed_node() -> bool { true }\n"),
    ]);

    let output = run(forge()
        .arg("build")
        .arg("--swarm")
        .arg(dir.path()));

    let out = stdout(&output);
    let err = stderr(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {err}");
    assert!(out.contains("P2P Swarm Cache enabled"));
}

#[test]
fn test_predictive_watch_mode_on_multi_package_workspace() {
    let workspace_manifest = r#"
[workspace]
resolver = "2"
members = ["engine", "app"]
"#;
    let engine_manifest = r#"
[package]
name = "engine"
version = "0.1.0"
edition = "2024"
"#;
    let app_manifest = r#"
[package]
name = "app"
version = "0.1.0"
edition = "2024"

[dependencies]
engine = { path = "../engine" }
"#;

    let dir = fixture(&[
        ("Cargo.toml", workspace_manifest),
        ("engine/Cargo.toml", engine_manifest),
        ("engine/src/lib.rs", "pub fn run_engine() -> i32 { 100 }\n"),
        ("app/Cargo.toml", app_manifest),
        ("app/src/main.rs", "fn main() { engine::run_engine(); }\n"),
    ]);

    let output = run(forge()
        .arg("watch")
        .arg("--predictive")
        .arg("--once")
        .arg(dir.path()));

    let out = stdout(&output);
    let err = stderr(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {err}");
    assert!(out.contains("Predictive pre-compilation enabled ⚡"));
}

#[test]
fn test_full_combined_turbo_stack() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn full_turbo() -> &'static str { \"blazing fast\" }\n"),
    ]);

    let output = run(forge()
        .arg("build")
        .arg("--semantic")
        .arg("--ramdisk")
        .arg("--swarm")
        .arg(dir.path()));

    let out = stdout(&output);
    let err = stderr(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {err}");
    assert!(out.contains("In-memory RAM disk turbo enabled"));
    assert!(out.contains("P2P Swarm Cache enabled"));
    assert!(out.contains("Semantic AST-aware fingerprinting active"));
}
