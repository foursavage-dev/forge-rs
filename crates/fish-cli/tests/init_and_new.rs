use std::process::Command;
use tempfile::tempdir;

fn fish_bin() -> String {
    env!("CARGO_BIN_EXE_fish").to_string()
}

#[test]
fn test_fish_init_in_empty_directory() {
    let temp = tempdir().unwrap();
    let status = Command::new(fish_bin())
        .arg("init")
        .arg("--path")
        .arg(temp.path())
        .status()
        .unwrap();

    assert!(status.success());
    let fish_yaml = temp.path().join("fish.yaml");
    assert!(fish_yaml.exists());

    let content = std::fs::read_to_string(fish_yaml).unwrap();
    assert!(content.contains("version: \"1\""));
    assert!(content.contains("build:"));
}

#[test]
fn test_fish_init_detects_rust_and_ts() {
    let temp = tempdir().unwrap();
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\n",
    )
    .unwrap();
    std::fs::write(
        temp.path().join("package.json"),
        "{\"name\": \"demo-web\"}\n",
    )
    .unwrap();

    let status = Command::new(fish_bin())
        .arg("init")
        .arg("--path")
        .arg(temp.path())
        .status()
        .unwrap();

    assert!(status.success());
    let content = std::fs::read_to_string(temp.path().join("fish.yaml")).unwrap();
    assert!(content.contains("rust-build:"));
    assert!(content.contains("ts-build:"));
}

#[test]
fn test_fish_new_rust_cli_project() {
    let temp = tempdir().unwrap();
    let project_dir = temp.path().join("test-app");

    let status = Command::new(fish_bin())
        .arg("new")
        .arg("test-app")
        .arg("--template")
        .arg("rust-cli")
        .arg("--path")
        .arg(&project_dir)
        .status()
        .unwrap();

    assert!(status.success());
    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("src/main.rs").exists());
    assert!(project_dir.join("fish.yaml").exists());
}

#[test]
fn test_fish_new_polyglot_monorepo() {
    let temp = tempdir().unwrap();
    let project_dir = temp.path().join("demo-monorepo");

    let status = Command::new(fish_bin())
        .arg("new")
        .arg("demo-monorepo")
        .arg("--template")
        .arg("polyglot-monorepo")
        .arg("--path")
        .arg(&project_dir)
        .status()
        .unwrap();

    assert!(status.success());
    assert!(project_dir.join("fish.yaml").exists());
    assert!(project_dir.join("services/api/Cargo.toml").exists());
    assert!(project_dir.join("services/worker/go.mod").exists());
    assert!(project_dir.join("apps/web/package.json").exists());
}

#[test]
fn test_fish_doctor_runs_successfully() {
    let status = Command::new(fish_bin()).arg("doctor").status().unwrap();

    assert!(status.success());
}
