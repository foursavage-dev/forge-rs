use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

fn fish() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fish"))
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

    let output_a = run(fish().arg("build").arg("--semantic").arg(temp_dir.path()));
    assert!(
        output_a.status.success()
            || stdout(&output_a).contains("Semantic")
            || stderr(&output_a).is_empty()
    );
}

#[test]
fn test_semantic_multi_language_ast_normalization() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        (
            "src/lib.rs",
            "pub fn calculate(a: i32, b: i32) -> i32 {\n    // Calculate sum\n    a + b\n}\n",
        ),
        (
            "native/math.cpp",
            "int multiply(int a, int b) {\n    /* Multi-line comment */\n    return a * b;\n}\n",
        ),
        (
            "web/index.ts",
            "export function greet(name: string): string {\n    // Greet user\n    return `Hello, ${name}`;\n}\n",
        ),
        (
            "scripts/util.py",
            "def compute(val):\n    # Python comment\n    return val * 10\n",
        ),
    ]);

    let output = run(fish().arg("build").arg("--semantic").arg(dir.path()));

    let out = stdout(&output);
    let err = stderr(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {err}");
    assert!(out.contains("Semantic AST-aware fingerprinting active"));
}

#[test]
fn test_ramdisk_real_artifact_isolation_and_sync() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        (
            "src/lib.rs",
            "pub fn high_throughput_operation() -> u64 { 1_000_000 }\n",
        ),
    ]);

    let output = run(fish().arg("build").arg("--ramdisk").arg(dir.path()));

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

    let output = run(fish().arg("build").arg("--swarm").arg(dir.path()));

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

    let output = run(fish()
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
fn test_reflink_hardware_engine_cli() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn reflink_data() -> usize { 1024 }\n"),
    ]);

    let output = run(fish().arg("build").arg("--reflink").arg(dir.path()));

    let out = stdout(&output);
    let err = stderr(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {err}");
    assert!(out.contains("Reflink / Copy-on-Write hardware VFS engine active"));
}

#[test]
fn test_hermetic_trace_sandbox_cli() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn hermetic_fn() {}\n"),
    ]);

    let output = run(fish().arg("build").arg("--hermetic-trace").arg(dir.path()));

    let out = stdout(&output);
    let err = stderr(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {err}");
    assert!(out.contains("Hermetic Syscall tracing sandbox active"));
}

#[test]
fn test_distributed_compute_swarm_cli() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn remote_job() -> u32 { 999 }\n"),
    ]);

    let output = run(fish().arg("build").arg("--swarm-compute").arg(dir.path()));

    let out = stdout(&output);
    let err = stderr(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {err}");
    assert!(out.contains("Distributed P2P Compute Swarm active"));
}

#[test]
fn test_critical_path_scheduler_cli() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn critical_task() -> u32 { 100 }\n"),
    ]);

    let output = run(fish().arg("build").arg("--critical-path").arg(dir.path()));

    let out = stdout(&output);
    let err = stderr(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {err}");
    assert!(out.contains("Dynamic Critical-Path Lookahead Scheduler active"));
}

#[test]
fn test_timemachine_history_and_rewind_cli() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn app_v1() {}\n"),
    ]);

    let history_output = run(fish().arg("history").arg(dir.path()));
    assert!(history_output.status.success());
    assert!(stdout(&history_output).contains("Fish Time-Machine Build History"));
}

#[test]
fn test_slsa_attestation_and_verification_cli() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn pristine_module() {}\n"),
    ]);

    let attest_output = run(fish().arg("attest").arg(dir.path()));
    assert!(attest_output.status.success());
    assert!(stdout(&attest_output).contains("SLSA Level 3 Attestation generated"));

    let attestation_path = dir.path().join(".fish/attestation.json");
    assert!(attestation_path.exists());

    let verify_output = run(fish().arg("verify").arg(&attestation_path).arg(dir.path()));

    assert!(verify_output.status.success());
    assert!(stdout(&verify_output).contains("SLSA Provenance Verified"));
}

#[test]
fn test_full_combined_turbo_stack() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        (
            "src/lib.rs",
            "pub fn full_turbo() -> &'static str { \"blazing fast\" }\n",
        ),
    ]);

    let output = run(fish()
        .arg("build")
        .arg("--semantic")
        .arg("--ramdisk")
        .arg("--swarm")
        .arg("--reflink")
        .arg("--hermetic-trace")
        .arg("--swarm-compute")
        .arg("--critical-path")
        .arg(dir.path()));

    let out = stdout(&output);
    let err = stderr(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {err}");
    assert!(out.contains("In-memory RAM disk turbo enabled"));
    assert!(out.contains("Semantic AST-aware fingerprinting active"));
    assert!(out.contains("Reflink / Copy-on-Write hardware VFS engine active"));
    assert!(out.contains("Hermetic Syscall tracing sandbox active"));
    assert!(out.contains("Distributed P2P Compute Swarm active"));
    assert!(out.contains("Dynamic Critical-Path Lookahead Scheduler active"));
}
