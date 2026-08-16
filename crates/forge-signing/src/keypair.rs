// Key pair management for signing operations

use crate::error::{SigningError, SigningResult};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

/// Options for key generation
#[derive(Debug, Clone, Default)]
pub struct KeyGenerationOptions {
    /// Whether to use a deterministic seed (for testing)
    pub deterministic: bool,
    /// Optional seed for deterministic generation
    pub seed: Option<[u8; 32]>,
}

/// Signing key pair
#[derive(Clone, Serialize, Deserialize)]
pub struct SigningKeyPair {
    #[serde(with = "serde_bytes")]
    public_key: Vec<u8>,
    #[serde(with = "serde_bytes")]
    secret_key: Vec<u8>,
}

impl SigningKeyPair {
    /// Generate a new random key pair
    pub fn generate(options: KeyGenerationOptions) -> SigningResult<Self> {
        let signing_key = if options.deterministic {
            if let Some(seed) = options.seed {
                SigningKey::from_bytes(&seed)
            } else {
                return Err(SigningError::KeyGenerationFailed(
                    "Deterministic generation requires a seed".to_string(),
                ));
            }
        } else {
            let mut bytes = [0u8; 32];
            use rand::RngCore;
            OsRng.fill_bytes(&mut bytes);
            SigningKey::from_bytes(&bytes)
        };

        let public_key: VerifyingKey = signing_key.verifying_key();
        let secret_key_bytes: [u8; 32] = signing_key.to_bytes();

        Ok(Self {
            public_key: public_key.to_bytes().to_vec(),
            secret_key: secret_key_bytes.to_vec(),
        })
    }

    /// Load key pair from file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> SigningResult<Self> {
        let content = fs::read_to_string(path.as_ref()).await?;
        let keypair: Self = serde_json::from_str(&content)?;
        Ok(keypair)
    }

    /// Save key pair to file
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> SigningResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path.as_ref(), content).await?;
        Ok(())
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.public_key
    }

    /// Get secret key bytes
    pub fn secret_key_bytes(&self) -> &[u8] {
        &self.secret_key
    }

    /// Convert to ed25519-dalek SigningKey
    pub fn to_signing_key(&self) -> SigningResult<SigningKey> {
        let secret_bytes: [u8; 32] =
            self.secret_key.clone().try_into().map_err(|_| {
                SigningError::InvalidKeyFormat("Invalid secret key length".to_string())
            })?;
        Ok(SigningKey::from_bytes(&secret_bytes))
    }

    /// Convert to ed25519-dalek VerifyingKey
    pub fn to_verifying_key(&self) -> SigningResult<VerifyingKey> {
        let public_bytes: [u8; 32] =
            self.public_key.clone().try_into().map_err(|_| {
                SigningError::InvalidKeyFormat("Invalid public key length".to_string())
            })?;
        VerifyingKey::from_bytes(&public_bytes)
            .map_err(|e| SigningError::InvalidKeyFormat(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_key_generation() {
        let keypair = SigningKeyPair::generate(KeyGenerationOptions::default()).unwrap();
        assert_eq!(keypair.public_key_bytes().len(), 32);
        assert_eq!(keypair.secret_key_bytes().len(), 32);
    }

    #[tokio::test]
    async fn test_key_persistence() {
        let keypair = SigningKeyPair::generate(KeyGenerationOptions::default()).unwrap();
        let temp_file = NamedTempFile::new().unwrap();
        keypair.save_to_file(temp_file.path()).await.unwrap();

        let loaded = SigningKeyPair::from_file(temp_file.path()).await.unwrap();
        assert_eq!(keypair.public_key_bytes(), loaded.public_key_bytes());
        assert_eq!(keypair.secret_key_bytes(), loaded.secret_key_bytes());
    }
}
