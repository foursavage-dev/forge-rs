// Forge Signing - Build Artifact Signing & Verification
// Provides cryptographic signing and verification for build artifacts
// with SBOM (Software Bill of Materials) generation

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod keypair;
pub mod sbom;
pub mod signature;
pub mod verifier;

pub use error::{SigningError, SigningResult};
pub use keypair::{KeyGenerationOptions, SigningKeyPair};
pub use sbom::{SbomFormat, SbomGenerator, SbomMetadata};
pub use signature::{ArtifactSignature, SignatureAlgorithm};
pub use verifier::{ArtifactVerifier, VerificationResult, VerificationStatus};

use std::path::Path;

/// Main signing service for build artifacts
#[derive(Clone)]
pub struct SigningService {
    keypair: SigningKeyPair,
    algorithm: SignatureAlgorithm,
}

impl SigningService {
    /// Create a new signing service with existing keypair
    pub fn new(keypair: SigningKeyPair) -> Self {
        Self {
            keypair,
            algorithm: SignatureAlgorithm::Ed25519,
        }
    }

    /// Create signing service from key file
    pub async fn from_key_file<P: AsRef<Path>>(private_key_path: P) -> SigningResult<Self> {
        let keypair = SigningKeyPair::from_file(private_key_path).await?;
        Ok(Self::new(keypair))
    }

    /// Sign an artifact with metadata
    pub async fn sign_artifact(
        &self,
        artifact_path: &Path,
        metadata: SbomMetadata,
    ) -> SigningResult<ArtifactSignature> {
        let signature =
            signature::sign_artifact(artifact_path, &self.keypair, self.algorithm, &metadata)
                .await?;
        Ok(signature)
    }

    /// Generate SBOM for a package
    pub async fn generate_sbom(
        &self,
        package_path: &Path,
        format: SbomFormat,
    ) -> SigningResult<String> {
        let generator = SbomGenerator::new(format);
        generator.generate(package_path).await
    }

    /// Get the public key for verification
    pub fn public_key(&self) -> &[u8] {
        self.keypair.public_key_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signing_service_creation() {
        let keypair = SigningKeyPair::generate(KeyGenerationOptions::default()).unwrap();
        let service = SigningService::new(keypair);
        assert!(!service.public_key().is_empty());
    }
}
