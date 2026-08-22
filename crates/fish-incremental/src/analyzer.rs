use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::change_detector::IncrementalChangeDetector;
use crate::ecosystem::{EcosystemType, detect_ecosystems, is_build_relevant_file};
use crate::patterns::{BuildPattern, PatternSeverity, PatternType};
use crate::suggestions::{OptimizationSuggestion, SuggestionPriority};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildAnalysis {
    pub project_path: String,
    pub analysis_timestamp: DateTime<Utc>,
    pub detected_ecosystems: Vec<EcosystemType>,
    pub patterns: Vec<BuildPattern>,
    pub suggestions: Vec<OptimizationSuggestion>,
    pub rebuild_frequency: f64,
    pub build_relevant_files_count: usize,
    pub ignored_files_count: usize,
}

#[derive(Clone, Default)]
pub struct IncrementalAnalyzer {
    change_detector: IncrementalChangeDetector,
}

impl IncrementalAnalyzer {
    pub fn new() -> Self {
        Self {
            change_detector: IncrementalChangeDetector::new(),
        }
    }

    pub fn change_detector(&self) -> &IncrementalChangeDetector {
        &self.change_detector
    }

    pub async fn analyze(
        &self,
        project_path: &str,
    ) -> Result<BuildAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        self.analyze_with_history(project_path, &[]).await
    }

    pub async fn analyze_with_history(
        &self,
        project_path: &str,
        file_changes: &[PathBuf],
    ) -> Result<BuildAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let root = Path::new(project_path);
        let ecosystems_info = detect_ecosystems(root);

        let mut detected_types: Vec<EcosystemType> =
            ecosystems_info.iter().map(|e| e.ecosystem).collect();
        detected_types.sort_by_key(|e| *e as u8);
        detected_types.dedup();

        let mut patterns = Vec::new();
        let mut suggestions = Vec::new();

        for eco in &ecosystems_info {
            if eco.lockfile_path.is_none() {
                patterns.push(BuildPattern {
                    pattern_type: PatternType::MissingLockfile,
                    severity: PatternSeverity::Warning,
                    description: format!(
                        "Lockfile missing for manifest at {}",
                        eco.manifest_path.display()
                    ),
                    affected_packages: vec![eco.manifest_path.to_string_lossy().to_string()],
                    frequency: 1.0,
                });

                suggestions.push(OptimizationSuggestion {
                    title: format!("Generate lockfile for {:?}", eco.ecosystem),
                    description: format!(
                        "Generating and committing a lockfile for {} ensures deterministic fingerprinting in incremental builds.",
                        eco.manifest_path.display()
                    ),
                    priority: SuggestionPriority::High,
                    estimated_impact: "Avoids unnecessary full dependency tree re-resolution".to_string(),
                    action_command: match eco.ecosystem {
                        EcosystemType::Rust => Some("cargo check".to_string()),
                        EcosystemType::TypeScript => Some("pnpm install || npm install".to_string()),
                        EcosystemType::Go => Some("go mod tidy".to_string()),
                        EcosystemType::Python => Some("poetry lock".to_string()),
                        _ => None,
                    },
                });
            }
        }

        if detected_types.len() > 1 {
            patterns.push(BuildPattern {
                pattern_type: PatternType::PolyglotBoundary,
                severity: PatternSeverity::Info,
                description: format!(
                    "Polyglot monorepo detected containing {} ecosystems: {:?}",
                    detected_types.len(),
                    detected_types
                ),
                affected_packages: ecosystems_info
                    .iter()
                    .map(|e| e.manifest_path.to_string_lossy().to_string())
                    .collect(),
                frequency: 1.0,
            });

            suggestions.push(OptimizationSuggestion {
                title: "Use DAG-aware Incremental affected builds".to_string(),
                description: "This project spans multiple languages. Running `fish affected` restricts build execution strictly to packages with invalidated inputs.".to_string(),
                priority: SuggestionPriority::Medium,
                estimated_impact: "Reduces cross-ecosystem build times by 40-75%".to_string(),
                action_command: Some("fish affected".to_string()),
            });
        }

        let mut relevant_count = 0;
        let mut ignored_count = 0;

        for change in file_changes {
            if is_build_relevant_file(change) {
                relevant_count += 1;
            } else {
                ignored_count += 1;
            }
        }

        let rebuild_freq = if !file_changes.is_empty() {
            (relevant_count as f64) / (file_changes.len() as f64)
        } else {
            0.0
        };

        if relevant_count > 0 && ignored_count > 0 {
            suggestions.push(OptimizationSuggestion {
                title: "Enable non-code change filter".to_string(),
                description: format!(
                    "Found {} non-code changes (docs/assets) out of {} total changes. Filtering these out saves unnecessary pipeline runs.",
                    ignored_count,
                    file_changes.len()
                ),
                priority: SuggestionPriority::Low,
                estimated_impact: "Eliminates redundant compilation cycles".to_string(),
                action_command: None,
            });
        }

        Ok(BuildAnalysis {
            project_path: project_path.to_string(),
            analysis_timestamp: Utc::now(),
            detected_ecosystems: detected_types,
            patterns,
            suggestions,
            rebuild_frequency: rebuild_freq,
            build_relevant_files_count: relevant_count,
            ignored_files_count: ignored_count,
        })
    }
}
