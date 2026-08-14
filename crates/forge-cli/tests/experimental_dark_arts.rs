use std::fs;
use std::process::{Command, Output};
use tempfile::{tempdir, TempDir};

const CARGO_MANIFEST: &str = r#"
[package]
name = "dark_arts_demo"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

fn forge() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_forge"));
    cmd.env("FORGE_CACHE_DIR", tempdir().unwrap().path());
    cmd
}

fn fixture(files: &[(&str, &str)]) -> TempDir {
    let dir = tempdir().unwrap();
    for &(path, content) in files {
        let full_path = dir.path().join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full_path, content).unwrap();
    }
    dir
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn run(cmd: &mut Command) -> Output {
    cmd.output().expect("failed to execute forge binary")
}

#[test]
fn test_experimental_turbolink_cli() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn fast_link() {}\n"),
    ]);

    let output = run(forge().arg("build").arg("--turbo-link").arg(dir.path()));
    let out = stdout(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {}", stderr(&output));
    assert!(out.contains("Linker Turbo-Hijack active"));
}

#[test]
fn test_experimental_speculative_cli() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn predict_code() {}\n"),
    ]);

    let output = run(forge().arg("build").arg("--speculative").arg(dir.path()));
    let out = stdout(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {}", stderr(&output));
    assert!(out.contains("Speculative Markov Pre-Compilation background engine active"));
}

#[test]
fn test_experimental_daemon_pool_cli() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn daemon_code() {}\n"),
    ]);

    let output = run(forge().arg("build").arg("--daemon-pool").arg(dir.path()));
    let out = stdout(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {}", stderr(&output));
    assert!(out.contains("Pre-Warmed Compiler Zombie-Daemon Pool active"));
}

#[test]
fn test_experimental_kernel_bypass_cli() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn dma_code() {}\n"),
    ]);

    let output = run(forge().arg("build").arg("--kernel-bypass").arg(dir.path()));
    let out = stdout(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {}", stderr(&output));
    assert!(out.contains("Kernel-Bypass Direct Ring-Buffer DMA VFS active"));
}

#[test]
fn test_experimental_wasm_sandbox_cli() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn wasm_code() {}\n"),
    ]);

    let output = run(forge().arg("build").arg("--wasm-sandbox").arg(dir.path()));
    let out = stdout(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {}", stderr(&output));
    assert!(out.contains("WASM / WASI Hermetic Plugin Sandbox active"));
}

#[test]
fn test_experimental_super_opt_cli() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn super_opt_code() {}\n"),
    ]);

    let output = run(forge().arg("build").arg("--super-opt").arg(dir.path()));
    let out = stdout(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {}", stderr(&output));
    assert!(out.contains("Autonomous Binary Super-Optimizer & AVX-512 Rewriter active"));
}

#[test]
fn test_experimental_jit_command() {
    let output = run(forge().arg("jit").arg("fast_compute_kernel").arg("100"));
    let out = stdout(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {}", stderr(&output));
    assert!(out.contains("In-Process Micro-JIT compiled `fast_compute_kernel`"));
}

#[test]
fn test_experimental_super_opt_command() {
    let dir = fixture(&[("app.bin", "RAW_BINARY_STREAM")]);
    let input = dir.path().join("app.bin");
    let output_file = dir.path().join("app_opt.bin");

    let output = run(forge().arg("super-opt").arg(&input).arg(&output_file));
    let out = stdout(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {}", stderr(&output));
    assert!(out.contains("Binary Super-Optimizer applied"));
    assert!(output_file.exists());
}

#[test]
fn test_experimental_live_patch_command() {
    let dir = fixture(&[
        ("target/release/app.exe", "BINARY_MACHINE_CODE"),
    ]);
    let target = dir.path().join("target/release/app.exe");

    let output = run(forge()
        .arg("live-patch")
        .arg("4321")
        .arg(&target)
        .arg(dir.path()));

    let out = stdout(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {}", stderr(&output));
    assert!(out.contains("Live Patch injected to PID 4321"));
}

#[test]
fn test_full_god_tier_experimental_stack() {
    let dir = fixture(&[
        ("Cargo.toml", CARGO_MANIFEST),
        ("src/lib.rs", "pub fn god_tier_mode() -> &'static str { \"unbounded speed\" }\n"),
    ]);

    let output = run(forge()
        .arg("build")
        .arg("--turbo-link")
        .arg("--speculative")
        .arg("--daemon-pool")
        .arg("--kernel-bypass")
        .arg("--wasm-sandbox")
        .arg("--super-opt")
        .arg(dir.path()));

    let out = stdout(&output);
    assert!(output.status.success(), "stdout: {out}, stderr: {}", stderr(&output));
    assert!(out.contains("Linker Turbo-Hijack active"));
    assert!(out.contains("Speculative Markov Pre-Compilation background engine active"));
    assert!(out.contains("Pre-Warmed Compiler Zombie-Daemon Pool active"));
    assert!(out.contains("Kernel-Bypass Direct Ring-Buffer DMA VFS active"));
    assert!(out.contains("WASM / WASI Hermetic Plugin Sandbox active"));
    assert!(out.contains("Autonomous Binary Super-Optimizer & AVX-512 Rewriter active"));
}
