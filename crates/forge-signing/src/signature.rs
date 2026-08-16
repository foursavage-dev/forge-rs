// Artifact signature creation and management

use crate::error::{SigningError, SigningResult};
use crate::keypair::SigningKeyPair;
use crate::sbom::SbomMetadata;
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;

/// Signature algorithms supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    Ed25519,
    #[allow(dead_code)]
    Rsa2048,
    #[allow(dead_code)]
    Rsa4096,
}

/// Artifact signature with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSignature {
    /// Signature algorithm used
    pub algorithm: SignatureAlgorithm,
    /// Base64-encoded signature
    pub signature: String,
    /// Hash of the artifact (SHA256)
    pub artifact_hash: String,
    /// Timestamp when signature was created
    pub timestamp: DateTime<Utc>,
    /// SBOM metadata
    pub metadata: SbomMetadata,
    /// Signer public key (for verification)
    pub signer_public_key: String,
}

/// Sign an artifact with the given keypair
pub async fn sign_artifact(
    artifact_path: &Path,
    keypair: &SigningKeyPair,
    algorithm: SignatureAlgorithm,
    metadata: &SbomMetadata,
) -> SigningResult<ArtifactSignature> {
    // Read artifact content
    let artifact_content = fs::read(artifact_path).await?;

    // Compute hash
    let mut hasher = Sha256::new();
    hasher.update(&artifact_content);
    let hash = hasher.finalize();
    let artifact_hash = general_purpose::STANDARD.encode(&hash);

    // Create signature based on algorithm
    let signature = match algorithm {
        SignatureAlgorithm::Ed25519 => {
            let signing_key = keypair.to_signing_key()?;
            let signature_bytes: Signature = signing_key.sign(&artifact_content);
            general_purpose::STANDARD.encode(signature_bytes.to_bytes())
        }
        _ => {
            return Err(SigningError::CryptoError(
                "Algorithm not yet implemented".to_string(),
            ));
        }
    };

    Ok(ArtifactSignature {
        algorithm,
        signature,
        artifact_hash,
        timestamp: Utc::now(),
        metadata: metadata.clone(),
        signer_public_key: general_purpose::STANDARD.encode(keypair.public_key_bytes()),
    })
}

/// Verify an artifact signature
pub fn verify_signature(
    artifact_content: &[u8],
    signature: &ArtifactSignature,
) -> SigningResult<bool> {
    match signature.algorithm {
        SignatureAlgorithm::Ed25519 => {
            let sig_bytes = general_purpose::STANDARD.decode(&signature.signature)?;
            let sig_array: [u8; 64] = sig_bytes.try_into().map_err(|_| {
                SigningError::SignatureVerificationFailed("Invalid signature length".to_string())
            })?;
            let signature_obj = Signature::from_bytes(&sig_array);
            let public_key_bytes =
                general_purpose::STANDARD.decode(&signature.signer_public_key)?;
            let public_key_array: [u8; 32] = public_key_bytes.try_into().map_err(|_| {
                SigningError::SignatureVerificationFailed("Invalid public key length".to_string())
            })?;
            let public_key = VerifyingKey::from_bytes(&public_key_array)
                .map_err(|error| SigningError::SignatureVerificationFailed(error.to_string()))?;

            Ok(public_key.verify(artifact_content, &signature_obj).is_ok())
        }
        _ => Err(SigningError::CryptoError(
            "Algorithm not yet implemented".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair::KeyGenerationOptions;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_artifact_signing() {
        let keypair = SigningKeyPair::generate(KeyGenerationOptions::default()).unwrap();
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), b"test artifact content")
            .await
            .unwrap();

        let metadata = SbomMetadata::default();
        let signature = sign_artifact(
            temp_file.path(),
            &keypair,
            SignatureAlgorithm::Ed25519,
            &metadata,
        )
        .await
        .unwrap();

        assert_eq!(signature.algorithm, SignatureAlgorithm::Ed25519);
        assert!(!signature.signature.is_empty());
    }

    #[tokio::test]
    async fn test_signature_verification() {
        let keypair = SigningKeyPair::generate(KeyGenerationOptions::default()).unwrap();
        let temp_file = NamedTempFile::new().unwrap();
        let content = b"test artifact content";
        fs::write(temp_file.path(), content).await.unwrap();

        let metadata = SbomMetadata::default();
        let signature = sign_artifact(
            temp_file.path(),
            &keypair,
            SignatureAlgorithm::Ed25519,
            &metadata,
        )
        .await
        .unwrap();

        let result = verify_signature(content, &signature).unwrap();
        assert!(result);
    }
}
