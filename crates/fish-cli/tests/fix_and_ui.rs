#![forbid(unsafe_code)]

use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

fn fish() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fish"))
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn test_fish_fix_runs_clean_on_healthy_project() {
    let output = fish().arg("fix").output().expect("failed to run forge fix");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("Fish Auto-Healer & AI Diagnostics"));
    assert!(text.contains("Clean") || text.contains("No compile errors"));
}

#[test]
fn test_fish_fix_detects_broken_code() {
    let dir = TempDir::new().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let cargo_toml = r#"
[package]
name = "broken_crate"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
    fs::write(dir.path().join("Cargo.toml"), cargo_toml).unwrap();

    let main_rs = r#"
fn main() {
    let x: u32 = "invalid_type";
}
"#;
    fs::write(src_dir.join("main.rs"), main_rs).unwrap();

    let output = fish()
        .arg("fix")
        .arg("--path")
        .arg(dir.path())
        .output()
        .expect("failed to run forge fix");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(
        text.contains("Diagnostics Summary")
            || text.contains("Mismatched Types")
            || text.contains("errors")
    );
}

#[test]
fn test_fish_ui_help_and_arguments() {
    let output = fish()
        .arg("ui")
        .arg("--help")
        .output()
        .expect("failed to run forge ui --help");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("--port"));
    assert!(text.contains("--open"));
}
