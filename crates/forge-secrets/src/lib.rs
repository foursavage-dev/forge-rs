// Forge Secrets - Secret Management Integration
// Secure secret injection for builds with Vault, AWS Secrets Manager, K8s secrets

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod manager;
pub mod vault;
pub mod aws;
pub mod kubernetes;

pub use manager::{SecretManager, Secret};
pub use vault::VaultSecretManager;
pub use aws::AwsSecretsManager;
pub use kubernetes::KubernetesSecretManager;

/// Main secrets service
pub struct SecretsService {
    manager: Box<dyn SecretManager>,
}

impl SecretsService {
    pub fn new(manager: Box<dyn SecretManager>) -> Self {
        Self { manager }
    }

    pub async fn get_secret(&self, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.manager.get_secret(key).await
    }

    pub async fn inject_secrets(&self, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.manager.inject_secrets(command).await
    }
}
