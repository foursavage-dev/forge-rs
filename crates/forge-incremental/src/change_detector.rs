use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::ecosystem::{EcosystemType, is_build_relevant_file};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeImpact {
    pub total_changed_files: usize,
    pub meaningful_changes: Vec<PathBuf>,
    pub ignored_files: Vec<PathBuf>,
    pub affected_ecosystems: Vec<EcosystemType>,
    pub requires_full_rebuild: bool,
}

#[derive(Debug, Clone, Default)]
pub struct IncrementalChangeDetector;

impl IncrementalChangeDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_changes<P: AsRef<Path>>(&self, changed_paths: &[P]) -> ChangeImpact {
        let mut meaningful = Vec::new();
        let mut ignored = Vec::new();
        let mut ecosystems = HashSet::new();
        let mut requires_full = false;

        for path_ref in changed_paths {
            let path = path_ref.as_ref();
            if is_build_relevant_file(path) {
                meaningful.push(path.to_path_buf());

                if let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                    && matches!(
                        file_name,
                        "Cargo.lock"
                            | "pnpm-lock.yaml"
                            | "yarn.lock"
                            | "package-lock.json"
                            | "go.sum"
                            | "poetry.lock"
                    )
                {
                    requires_full = true;
                }

                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext.to_lowercase().as_str() {
                        "rs" => {
                            ecosystems.insert(EcosystemType::Rust);
                        }
                        "ts" | "tsx" | "js" | "jsx" | "json" => {
                            ecosystems.insert(EcosystemType::TypeScript);
                        }
                        "go" => {
                            ecosystems.insert(EcosystemType::Go);
                        }
                        "py" => {
                            ecosystems.insert(EcosystemType::Python);
                        }
                        "java" | "kt" | "kts" => {
                            ecosystems.insert(EcosystemType::Java);
                        }
                        "cs" | "fs" => {
                            ecosystems.insert(EcosystemType::DotNet);
                        }
                        "c" | "cpp" | "cc" | "h" | "hpp" => {
                            ecosystems.insert(EcosystemType::Cpp);
                        }
                        _ => {}
                    }
                }
            } else {
                ignored.push(path.to_path_buf());
            }
        }

        ChangeImpact {
            total_changed_files: changed_paths.len(),
            meaningful_changes: meaningful,
            ignored_files: ignored,
            affected_ecosystems: ecosystems.into_iter().collect(),
            requires_full_rebuild: requires_full,
        }
    }
}
