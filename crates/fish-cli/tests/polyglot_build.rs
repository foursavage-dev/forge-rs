use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn fish() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fish"))
}

#[test]
fn test_polyglot_rust_and_typescript_build() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let cargo_toml = root.join("Cargo.toml");
    fs::write(
        &cargo_toml,
        r#"[package]
name = "polyglot_rust"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();
    let rust_src = root.join("src");
    fs::create_dir_all(&rust_src).unwrap();
    fs::write(rust_src.join("lib.rs"), "pub fn hello() {}").unwrap();

    let package_json = root.join("package.json");
    fs::write(
        &package_json,
        r#"{
  "name": "polyglot_ts",
  "version": "1.0.0",
  "scripts": {
    "build": "echo ts_built"
  }
}
"#,
    )
    .unwrap();

    let output = fish()
        .arg("build")
        .arg(root)
        .arg("--no-cache")
        .output()
        .unwrap();

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());
}
