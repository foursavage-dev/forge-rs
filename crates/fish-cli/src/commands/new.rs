use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub enum ProjectTemplate {
    RustCli,
    RustLib,
    PolyglotMonorepo,
    Fullstack,
    CppApp,
}

impl ProjectTemplate {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust-cli" | "rust" | "cli" => Some(ProjectTemplate::RustCli),
            "rust-lib" | "lib" => Some(ProjectTemplate::RustLib),
            "polyglot-monorepo" | "monorepo" => Some(ProjectTemplate::PolyglotMonorepo),
            "fullstack" | "web" => Some(ProjectTemplate::Fullstack),
            "cpp-app" | "cpp" | "c" => Some(ProjectTemplate::CppApp),
            _ => None,
        }
    }
}

pub fn create_rust_cli(dir: &Path, name: &str) -> std::io::Result<()> {
    fs::create_dir_all(dir.join("src"))?;

    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = {{ version = "4.5", features = ["derive"] }}
"#
    );
    fs::write(dir.join("Cargo.toml"), cargo_toml)?;

    let main_rs = r#"use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    name: Option<String>,
}

fn main() {
    let args = Args::parse();
    let recipient = args.name.as_deref().unwrap_or("World");
    println!("Hello, {}! Powered by Fish.", recipient);
}
"#;
    fs::write(dir.join("src/main.rs"), main_rs)?;

    let fish_yaml = r#"version: "1"

tasks:
  build:
    command: cargo build --release
    cache:
      enabled: true
      outputs:
        - target/release

  test:
    command: cargo test
    depends_on:
      - build

  check:
    command: cargo clippy -- -D warnings
"#;
    fs::write(dir.join("fish.yaml"), fish_yaml)?;

    Ok(())
}

pub fn create_polyglot_monorepo(dir: &Path, name: &str) -> std::io::Result<()> {
    fs::create_dir_all(dir.join("services/api/src"))?;
    fs::create_dir_all(dir.join("services/worker"))?;
    fs::create_dir_all(dir.join("apps/web/src"))?;

    let root_fish = r#"version: "1"

tasks:
  api-build:
    cwd: services/api
    command: cargo build --release
    cache:
      enabled: true

  api-test:
    cwd: services/api
    command: cargo test
    depends_on:
      - api-build

  worker-build:
    cwd: services/worker
    command: go build -o worker .
    cache:
      enabled: true

  worker-test:
    cwd: services/worker
    command: go test ./...
    depends_on:
      - worker-build

  web-build:
    cwd: apps/web
    command: npm run build
    cache:
      enabled: true

  web-test:
    cwd: apps/web
    command: npm test
    depends_on:
      - web-build

  all:
    depends_on:
      - api-build
      - worker-build
      - web-build

  test:
    depends_on:
      - api-test
      - worker-test
"#;
    fs::write(dir.join("fish.yaml"), root_fish)?;

    let api_cargo = r#"[package]
name = "api-service"
version = "0.1.0"
edition = "2024"

[dependencies]
tokio = { version = "1", features = ["full"] }
"#;
    fs::write(dir.join("services/api/Cargo.toml"), api_cargo)?;
    fs::write(
        dir.join("services/api/src/main.rs"),
        "#[tokio::main]\nasync fn main() {\n    println!(\"API Service running on port 8080\");\n}\n",
    )?;

    let go_mod = "module worker\n\ngo 1.21\n";
    fs::write(dir.join("services/worker/go.mod"), go_mod)?;
    let go_main =
        "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"Worker running\")\n}\n";
    fs::write(dir.join("services/worker/main.go"), go_main)?;

    let pkg_json = format!(
        r#"{{
  "name": "{name}-web",
  "version": "0.1.0",
  "scripts": {{
    "build": "echo 'Building web app...'",
    "test": "echo 'Testing web app...'"
  }}
}}
"#
    );
    fs::write(dir.join("apps/web/package.json"), pkg_json)?;

    Ok(())
}

pub fn run_new(name: &str, template: Option<&str>, path: Option<PathBuf>) -> ExitCode {
    let target_dir = path.unwrap_or_else(|| PathBuf::from(name));

    if target_dir.exists() {
        eprintln!(
            "error: target directory '{}' already exists",
            target_dir.display()
        );
        return ExitCode::FAILURE;
    }

    let tpl = template
        .and_then(ProjectTemplate::parse)
        .unwrap_or(ProjectTemplate::RustCli);

    println!(
        "🦀 Creating new Fish project '{}' in {}",
        name,
        target_dir.display()
    );

    let res = match tpl {
        ProjectTemplate::RustCli
        | ProjectTemplate::RustLib
        | ProjectTemplate::Fullstack
        | ProjectTemplate::CppApp => create_rust_cli(&target_dir, name),
        ProjectTemplate::PolyglotMonorepo => create_polyglot_monorepo(&target_dir, name),
    };

    match res {
        Ok(_) => {
            println!("  [ok] Project scaffolding complete!");
            println!("\nNext steps:");
            println!("  cd {}", target_dir.display());
            println!("  fish build");
            println!("  fish test");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: failed to scaffold project: {err}");
            ExitCode::FAILURE
        }
    }
}
