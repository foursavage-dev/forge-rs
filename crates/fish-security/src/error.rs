// Error types for security operations

use thiserror::Error;

/// Result type for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Errors that can occur during security scanning
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Lock file not found: {0}")]
    LockFileNotFound(String),

    #[error("Lock file parse failed: {0}")]
    LockFileParseFailed(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("No vulnerabilities found")]
    NoVulnerabilitiesFound,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}
