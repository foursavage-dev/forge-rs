#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod analyzer;
pub mod change_detector;
pub mod ecosystem;
pub mod patterns;
pub mod suggestions;

pub use analyzer::{BuildAnalysis, IncrementalAnalyzer};
pub use change_detector::{ChangeImpact, IncrementalChangeDetector};
pub use ecosystem::{EcosystemInfo, EcosystemType, detect_ecosystems, is_build_relevant_file};
pub use patterns::{BuildPattern, PatternSeverity, PatternType};
pub use suggestions::{OptimizationSuggestion, SuggestionPriority};

use std::path::PathBuf;

pub struct IncrementalAnalysisService {
    analyzer: IncrementalAnalyzer,
}

impl IncrementalAnalysisService {
    pub fn new() -> Self {
        Self {
            analyzer: IncrementalAnalyzer::new(),
        }
    }

    pub fn analyzer(&self) -> &IncrementalAnalyzer {
        &self.analyzer
    }

    pub async fn analyze(
        &self,
        project_path: &str,
    ) -> Result<BuildAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        self.analyzer.analyze(project_path).await
    }

    pub async fn analyze_with_history(
        &self,
        project_path: &str,
        file_changes: &[PathBuf],
    ) -> Result<BuildAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        self.analyzer
            .analyze_with_history(project_path, file_changes)
            .await
    }
}

impl Default for IncrementalAnalysisService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_build_relevant_file_filter() {
        assert!(is_build_relevant_file(Path::new("src/main.rs")));
        assert!(is_build_relevant_file(Path::new("Cargo.toml")));
        assert!(is_build_relevant_file(Path::new("packages/app/index.ts")));
        assert!(is_build_relevant_file(Path::new("cmd/server/main.go")));

        assert!(!is_build_relevant_file(Path::new("README.md")));
        assert!(!is_build_relevant_file(Path::new(
            "docs/architecture.markdown"
        )));
        assert!(!is_build_relevant_file(Path::new("LICENSE")));
        assert!(!is_build_relevant_file(Path::new(".gitignore")));
        assert!(!is_build_relevant_file(Path::new(
            ".github/workflows/ci.yml"
        )));
        assert!(!is_build_relevant_file(Path::new("assets/logo.png")));
        assert!(!is_build_relevant_file(Path::new("target/debug/app.exe")));
        assert!(!is_build_relevant_file(Path::new(
            "node_modules/pkg/index.js"
        )));
    }

    #[test]
    fn test_change_detector_ecosystem_identification() {
        let detector = IncrementalChangeDetector::new();
        let changes = vec![
            PathBuf::from("crates/core/src/lib.rs"),
            PathBuf::from("frontend/src/App.tsx"),
            PathBuf::from("docs/guide.md"),
            PathBuf::from("Cargo.lock"),
        ];

        let impact = detector.analyze_changes(&changes);
        assert_eq!(impact.total_changed_files, 4);
        assert_eq!(impact.meaningful_changes.len(), 3);
        assert_eq!(impact.ignored_files.len(), 1);
        assert!(impact.requires_full_rebuild);
        assert!(impact.affected_ecosystems.contains(&EcosystemType::Rust));
        assert!(
            impact
                .affected_ecosystems
                .contains(&EcosystemType::TypeScript)
        );
    }

    #[tokio::test]
    async fn test_incremental_service_analysis() {
        let service = IncrementalAnalysisService::new();
        let changes = vec![
            PathBuf::from("crates/forge-core/src/lib.rs"),
            PathBuf::from("README.md"),
        ];

        let analysis = service.analyze_with_history(".", &changes).await.unwrap();

        assert!(!analysis.project_path.is_empty());
        assert_eq!(analysis.build_relevant_files_count, 1);
        assert_eq!(analysis.ignored_files_count, 1);
        assert_eq!(analysis.rebuild_frequency, 0.5);
    }
}
