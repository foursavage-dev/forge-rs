// Error types for signing operations

use thiserror::Error;

/// Result type for signing operations
pub type SigningResult<T> = Result<T, SigningError>;

/// Errors that can occur during signing/verification
#[derive(Error, Debug)]
pub enum SigningError {
    #[error("Key file not found: {0}")]
    KeyFileNotFound(String),

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("Artifact not found: {0}")]
    ArtifactNotFound(String),

    #[error("Artifact read failed: {0}")]
    ArtifactReadFailed(String),

    #[error("Signature creation failed: {0}")]
    SignatureCreationFailed(String),

    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    #[error("SBOM generation failed: {0}")]
    SbomGenerationFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
}
