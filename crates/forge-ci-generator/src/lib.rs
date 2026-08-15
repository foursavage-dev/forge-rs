#![forbid(unsafe_code)]

pub mod github;
pub mod gitlab;
pub mod circleci;
pub mod bitbucket;
pub mod matrix;

pub use github::GitHubActionsGenerator;
pub use gitlab::GitLabCIGenerator;
pub use circleci::CircleCIGenerator;
pub use bitbucket::BitbucketPipelineGenerator;
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
    CircleCI,
    BitbucketPipelines,
    All,
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

impl CIConfig {
    pub fn with_platform(mut self, platform: CIPlatform) -> Self {
        self.platform = platform;
        self
    }
    
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }
    
    pub fn with_remote_cache(mut self, url: String) -> Self {
        self.remote_cache_url = Some(url);
        self
    }
    
    pub fn generate_ci(&self, matrix: &CIMatrix) -> Result<String> {
        match self.platform {
            CIPlatform::GitHubActions => {
                let generator = GitHubActionsGenerator::new(self.clone());
                generator.generate_workflow(matrix)
            }
            CIPlatform::GitLabCI => {
                let generator = GitLabCIGenerator::new(self.clone());
                generator.generate_pipeline(matrix)
            }
            CIPlatform::CircleCI => {
                let generator = CircleCIGenerator::new(self.clone());
                generator.generate_config(matrix)
            }
            CIPlatform::BitbucketPipelines => {
                let generator = BitbucketPipelineGenerator::new(self.clone());
                generator.generate_config(matrix)
            }
            CIPlatform::All => {
                // Generate all configurations and return them as a combined output
                let mut output = String::new();
                
                let github_gen = GitHubActionsGenerator::new(self.clone());
                output.push_str(&github_gen.generate_workflow(matrix)?);
                output.push_str("\n---\n\n");
                
                let gitlab_gen = GitLabCIGenerator::new(self.clone());
                output.push_str(&gitlab_gen.generate_pipeline(matrix)?);
                output.push_str("\n---\n\n");
                
                let circle_gen = CircleCIGenerator::new(self.clone());
                output.push_str(&circle_gen.generate_config(matrix)?);
                output.push_str("\n---\n\n");
                
                let bitbucket_gen = BitbucketPipelineGenerator::new(self.clone());
                output.push_str(&bitbucket_gen.generate_config(matrix)?);
                
                Ok(output)
            }
        }
    }
}
