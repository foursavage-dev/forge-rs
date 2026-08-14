#![forbid(unsafe_code)]

pub mod github;
pub mod gitlab;
pub mod matrix;

pub use github::GitHubActionsGenerator;
pub use gitlab::GitLabCIGenerator;
pub use matrix::{CIMatrix, CIJob, CacheConfig};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CIGeneratorError {
    #[error("Failed to generate CI configuration: {0}")]
    GenerationError(String),
    
    #[error("Template error: {0}")]
    TemplateError(String),
    
    #[error("Analysis error: {0}")]
    AnalysisError(String),
}

impl From<handlebars::TemplateError> for CIGeneratorError {
    fn from(e: handlebars::TemplateError) -> Self {
        Self::TemplateError(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CIGeneratorError>;

#[derive(Debug, Clone)]
pub struct CIConfig {
    pub platform: CIPlatform,
    pub cache_enabled: bool,
    pub remote_cache_url: Option<String>,
    pub jobs_per_run: usize,
    pub timeout_minutes: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CIPlatform {
    GitHubActions,
    GitLabCI,
    Both,
}

impl Default for CIConfig {
    fn default() -> Self {
        Self {
            platform: CIPlatform::GitHubActions,
            cache_enabled: true,
            remote_cache_url: None,
            jobs_per_run: 4,
            timeout_minutes: 30,
        }
    }
}
