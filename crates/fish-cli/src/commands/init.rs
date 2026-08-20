use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub struct DetectedLanguage {
    pub name: &'static str,
    pub backend: &'static str,
    pub build_cmd: &'static str,
    pub test_cmd: &'static str,
}

pub fn detect_workspace_languages(dir: &Path) -> Vec<DetectedLanguage> {
    let mut detected = Vec::new();

    if dir.join("Cargo.toml").exists() {
        detected.push(DetectedLanguage {
            name: "Rust",
            backend: "rust",
            build_cmd: "cargo build",
            test_cmd: "cargo test",
        });
    }

    if dir.join("package.json").exists() {
        detected.push(DetectedLanguage {
            name: "TypeScript / Node",
            backend: "ts",
            build_cmd: "npm run build",
            test_cmd: "npm test",
        });
    }

    if dir.join("go.mod").exists() {
        detected.push(DetectedLanguage {
            name: "Go",
            backend: "go",
            build_cmd: "go build ./...",
            test_cmd: "go test ./...",
        });
    }

    if dir.join("CMakeLists.txt").exists() || dir.join("Makefile").exists() {
        detected.push(DetectedLanguage {
            name: "C / C++",
            backend: "cc",
            build_cmd: "cmake -B build && cmake --build build",
            test_cmd: "ctest --test-dir build",
        });
    }

    if dir.join("pyproject.toml").exists() || dir.join("requirements.txt").exists() {
        detected.push(DetectedLanguage {
            name: "Python",
            backend: "py",
            build_cmd: "python -m build",
            test_cmd: "pytest",
        });
    }

    if dir.join("pom.xml").exists() || dir.join("build.gradle").exists() {
        detected.push(DetectedLanguage {
            name: "Java",
            backend: "java",
            build_cmd: "mvn compile",
            test_cmd: "mvn test",
        });
    }

    if dir.join("Package.swift").exists() {
        detected.push(DetectedLanguage {
            name: "Swift",
            backend: "swift",
            build_cmd: "swift build",
            test_cmd: "swift test",
        });
    }

    if dir.join("pubspec.yaml").exists() {
        detected.push(DetectedLanguage {
            name: "Dart",
            backend: "dart",
            build_cmd: "dart compile exe bin/main.dart",
            test_cmd: "dart test",
        });
    }

    if dir.join("build.zig").exists() {
        detected.push(DetectedLanguage {
            name: "Zig",
            backend: "zig",
            build_cmd: "zig build",
            test_cmd: "zig build test",
        });
    }

    if dir.join("Dockerfile").exists() {
        detected.push(DetectedLanguage {
            name: "Docker",
            backend: "docker",
            build_cmd: "docker build .",
            test_cmd: "docker run --rm test",
        });
    }

    detected
}

pub fn generate_fish_yaml(languages: &[DetectedLanguage]) -> String {
    let mut out = String::from("version: \"1\"\n\ntasks:\n");

    if languages.is_empty() {
        out.push_str("  build:\n    command: echo \"Building project...\"\n\n");
        out.push_str(
            "  test:\n    command: echo \"Running tests...\"\n    depends_on:\n      - build\n",
        );
        return out;
    }

    for lang in languages {
        let prefix = lang.backend;
        out.push_str(&format!("  {prefix}-build:\n"));
        out.push_str(&format!("    command: {}\n", lang.build_cmd));
        out.push_str("    cache:\n      enabled: true\n\n");

        out.push_str(&format!("  {prefix}-test:\n"));
        out.push_str(&format!("    command: {}\n", lang.test_cmd));
        out.push_str(&format!("    depends_on:\n      - {prefix}-build\n\n"));
    }

    out.push_str("  build:\n    depends_on:\n");
    for lang in languages {
        out.push_str(&format!("      - {}-build\n", lang.backend));
    }
    out.push_str("\n  test:\n    depends_on:\n");
    for lang in languages {
        out.push_str(&format!("      - {}-test\n", lang.backend));
    }

    out
}

pub fn run_init(path: Option<PathBuf>, force: bool) -> ExitCode {
    let target_dir = path.unwrap_or_else(|| PathBuf::from("."));
    let fish_file = target_dir.join("fish.yaml");

    if fish_file.exists() && !force {
        eprintln!(
            "error: forge.yaml already exists in {}. Use --force to overwrite.",
            target_dir.display()
        );
        return ExitCode::FAILURE;
    }

    println!("🦀 Initializing Fish in {}", target_dir.display());
    let detected = detect_workspace_languages(&target_dir);

    if detected.is_empty() {
        println!(
            "  [info] No specific language manifest detected. Generating default configuration."
        );
    } else {
        println!("  [ok] Detected {} language(s):", detected.len());
        for lang in &detected {
            println!("    - {} (backend: {})", lang.name, lang.backend);
        }
    }

    let config_content = generate_fish_yaml(&detected);
    if let Err(e) = std::fs::write(&fish_file, config_content) {
        eprintln!("error: failed to write forge.yaml: {e}");
        return ExitCode::FAILURE;
    }

    println!("  [ok] Successfully generated {}", fish_file.display());
    println!("\nYou can now run:");
    println!("  forge build    # to build all targets");
    println!("  forge test     # to execute test suites");
    println!("  forge graph    # to visualize task dependency graph");

    ExitCode::SUCCESS
}
