use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EcosystemType {
    Rust,
    TypeScript,
    Go,
    Python,
    Java,
    DotNet,
    Cpp,
    Docker,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemInfo {
    pub ecosystem: EcosystemType,
    pub manifest_path: PathBuf,
    pub lockfile_path: Option<PathBuf>,
    pub package_name: Option<String>,
}

pub fn is_build_relevant_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    if path_str.contains(".git")
        || path_str.contains(".github")
        || path_str.contains(".gitlab")
        || path_str.contains(".vscode")
        || path_str.contains(".idea")
        || path_str.contains("target")
        || path_str.contains("node_modules")
        || path_str.contains(".venv")
        || path_str.contains("__pycache__")
        || path_str.contains("dist")
        || path_str.contains("build")
    {
        return false;
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    if file_name.starts_with(".gitignore")
        || file_name.starts_with(".gitattributes")
        || file_name.starts_with(".editorconfig")
        || file_name.starts_with(".prettier")
        || file_name.starts_with(".eslint")
        || file_name.starts_with("license")
        || file_name.starts_with("contributing")
        || file_name.starts_with("changelog")
    {
        return false;
    }

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        !matches!(
            ext_lower.as_str(),
            "md" | "markdown" | "txt" | "rst" | "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico"
        )
    } else {
        matches!(
            file_name.as_str(),
            "dockerfile" | "makefile" | "gemfile" | "rakefile"
        )
    }
}

pub fn detect_ecosystems(root: &Path) -> Vec<EcosystemInfo> {
    let mut results = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        let mut subdirs = Vec::new();
        let mut has_cargo = None;
        let mut has_cargo_lock = None;
        let mut has_package_json = None;
        let mut has_pnpm_lock = None;
        let mut has_yarn_lock = None;
        let mut has_npm_lock = None;
        let mut has_go_mod = None;
        let mut has_go_sum = None;
        let mut has_pyproject = None;
        let mut has_poetry_lock = None;
        let mut has_pom = None;
        let mut has_gradle = None;
        let mut has_cmake = None;
        let mut has_docker = None;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str != "target"
                    && name_str != "node_modules"
                    && name_str != ".git"
                    && name_str != ".venv"
                    && name_str != "__pycache__"
                    && name_str != "dist"
                    && name_str != "build"
                {
                    subdirs.push(path);
                }
            } else if path.is_file()
                && let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            {
                match file_name {
                    "Cargo.toml" => has_cargo = Some(path.clone()),
                    "Cargo.lock" => has_cargo_lock = Some(path.clone()),
                    "package.json" => has_package_json = Some(path.clone()),
                    "pnpm-lock.yaml" => has_pnpm_lock = Some(path.clone()),
                    "yarn.lock" => has_yarn_lock = Some(path.clone()),
                    "package-lock.json" => has_npm_lock = Some(path.clone()),
                    "go.mod" => has_go_mod = Some(path.clone()),
                    "go.sum" => has_go_sum = Some(path.clone()),
                    "pyproject.toml" => has_pyproject = Some(path.clone()),
                    "poetry.lock" => has_poetry_lock = Some(path.clone()),
                    "pom.xml" => has_pom = Some(path.clone()),
                    "build.gradle" | "build.gradle.kts" => has_gradle = Some(path.clone()),
                    "CMakeLists.txt" => has_cmake = Some(path.clone()),
                    "Dockerfile" => has_docker = Some(path.clone()),
                    _ => {}
                }
            }
        }

        if let Some(manifest) = has_cargo {
            results.push(EcosystemInfo {
                ecosystem: EcosystemType::Rust,
                manifest_path: manifest,
                lockfile_path: has_cargo_lock,
                package_name: None,
            });
        }
        if let Some(manifest) = has_package_json {
            let lockfile = has_pnpm_lock.or(has_yarn_lock).or(has_npm_lock);
            results.push(EcosystemInfo {
                ecosystem: EcosystemType::TypeScript,
                manifest_path: manifest,
                lockfile_path: lockfile,
                package_name: None,
            });
        }
        if let Some(manifest) = has_go_mod {
            results.push(EcosystemInfo {
                ecosystem: EcosystemType::Go,
                manifest_path: manifest,
                lockfile_path: has_go_sum,
                package_name: None,
            });
        }
        if let Some(manifest) = has_pyproject {
            results.push(EcosystemInfo {
                ecosystem: EcosystemType::Python,
                manifest_path: manifest,
                lockfile_path: has_poetry_lock,
                package_name: None,
            });
        }
        if let Some(manifest) = has_pom.or(has_gradle) {
            results.push(EcosystemInfo {
                ecosystem: EcosystemType::Java,
                manifest_path: manifest,
                lockfile_path: None,
                package_name: None,
            });
        }
        if let Some(manifest) = has_cmake {
            results.push(EcosystemInfo {
                ecosystem: EcosystemType::Cpp,
                manifest_path: manifest,
                lockfile_path: None,
                package_name: None,
            });
        }
        if let Some(manifest) = has_docker {
            results.push(EcosystemInfo {
                ecosystem: EcosystemType::Docker,
                manifest_path: manifest,
                lockfile_path: None,
                package_name: None,
            });
        }

        for sub in subdirs {
            stack.push(sub);
        }
    }

    results
}
