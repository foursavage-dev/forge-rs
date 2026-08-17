#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CasError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Artifact not found: {0}")]
    ArtifactNotFound(String),

    #[error("Storage backend error: {0}")]
    BackendError(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Hash computation error: {0}")]
    Hash(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Operation cancelled")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, CasError>;
