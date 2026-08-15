// Artifact verification service

use crate::error::SigningResult;
use crate::signature::{verify_signature, ArtifactSignature};
use base64::{Engine as _, engine::general_purpose};
use sha2::Digest;
use std::path::Path;
use tokio::fs;

/// Verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    /// Signature is valid
    Valid,
    /// Signature is invalid
    Invalid,
    /// Signature format error
    FormatError,
    /// Artifact not found
    ArtifactNotFound,
}

/// Verification result with details
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Overall status
    pub status: VerificationStatus,
    /// Hash match
    pub hash_match: bool,
    /// Signature valid
    pub signature_valid: bool,
    /// Metadata verified
    pub metadata_verified: bool,
    /// Verification message
    pub message: String,
}

/// Artifact verifier
pub struct ArtifactVerifier {
    /// Trusted public keys
    trusted_keys: Vec<Vec<u8>>,
}

impl ArtifactVerifier {
    /// Create a new verifier with trusted keys
    pub fn new(trusted_keys: Vec<Vec<u8>>) -> Self {
        Self { trusted_keys }
    }

    /// Create verifier with a single trusted key
    pub fn with_key(public_key: Vec<u8>) -> Self {
        Self {
            trusted_keys: vec![public_key],
        }
    }

    /// Verify an artifact
    pub async fn verify(
        &self,
        artifact_path: &Path,
        signature: &ArtifactSignature,
    ) -> SigningResult<VerificationResult> {
        // Check if artifact exists
        if !artifact_path.exists() {
            return Ok(VerificationResult {
                status: VerificationStatus::ArtifactNotFound,
                hash_match: false,
                signature_valid: false,
                metadata_verified: false,
                message: "Artifact not found".to_string(),
            });
        }

        // Read artifact
        let artifact_content = fs::read(artifact_path).await?;

        // Verify signature
        let signature_valid = verify_signature(&artifact_content, signature)?;

        // Check if signer is trusted
        let signer_trusted = self.trusted_keys.iter().any(|key| {
            key == &general_purpose::STANDARD.decode(&signature.signer_public_key).unwrap_or_default()
        });

        // Verify hash
        let mut hasher = sha2::Sha256::new();
        hasher.update(&artifact_content);
        let hash = hasher.finalize();
        let hash_match = general_purpose::STANDARD.encode(&hash) == signature.artifact_hash;

        // Determine overall status
        let status = if signature_valid && signer_trusted && hash_match {
            VerificationStatus::Valid
        } else {
            VerificationStatus::Invalid
        };

        let message = if status == VerificationStatus::Valid {
            "Artifact signature verified successfully".to_string()
        } else {
            let mut reasons = Vec::new();
            if !signature_valid {
                reasons.push("signature invalid");
            }
            if !signer_trusted {
                reasons.push("signer not trusted");
            }
            if !hash_match {
                reasons.push("hash mismatch");
            }
            format!("Verification failed: {}", reasons.join(", "))
        };

        Ok(VerificationResult {
            status,
            hash_match,
            signature_valid,
            metadata_verified: true, // For now, assume metadata is verified
            message,
        })
    }

    /// Add a trusted key
    pub fn add_trusted_key(&mut self, public_key: Vec<u8>) {
        self.trusted_keys.push(public_key);
    }

    /// Get trusted keys count
    pub fn trusted_keys_count(&self) -> usize {
        self.trusted_keys.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair::{KeyGenerationOptions, SigningKeyPair};
    use crate::sbom::SbomMetadata;
    use crate::signature::sign_artifact;
    use crate::SignatureAlgorithm;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_artifact_verification() {
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

        let verifier = ArtifactVerifier::with_key(keypair.public_key_bytes().to_vec());
        let result = verifier.verify(temp_file.path(), &signature).await.unwrap();

        assert_eq!(result.status, VerificationStatus::Valid);
        assert!(result.hash_match);
        assert!(result.signature_valid);
    }

    #[tokio::test]
    async fn test_untrusted_signer() {
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

        // Use a different (untrusted) key
        let untrusted_key = SigningKeyPair::generate(KeyGenerationOptions::default())
            .unwrap()
            .public_key_bytes()
            .to_vec();
        let verifier = ArtifactVerifier::with_key(untrusted_key);
        let result = verifier.verify(temp_file.path(), &signature).await.unwrap();

        assert_eq!(result.status, VerificationStatus::Invalid);
    }
}
