// Forge Incremental - Incremental Build Analysis
// Analyzes incremental build patterns and suggests optimizations

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod analyzer;
pub mod patterns;
pub mod suggestions;

pub use analyzer::{BuildAnalysis, IncrementalAnalyzer};
pub use patterns::{BuildPattern, PatternType};
pub use suggestions::{OptimizationSuggestion, SuggestionPriority};

/// Main incremental analysis service
pub struct IncrementalAnalysisService {
    analyzer: IncrementalAnalyzer,
}

impl IncrementalAnalysisService {
    pub fn new() -> Self {
        Self {
            analyzer: IncrementalAnalyzer::new(),
        }
    }

    pub async fn analyze(
        &self,
        project_path: &str,
    ) -> Result<BuildAnalysis, Box<dyn std::error::Error>> {
        self.analyzer.analyze(project_path).await
    }
}

impl Default for IncrementalAnalysisService {
    fn default() -> Self {
        Self::new()
    }
}
