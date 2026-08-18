// Forge Secrets - Secret Management Integration
// Secure secret injection for builds with Vault, AWS Secrets Manager, K8s secrets

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod aws;
pub mod kubernetes;
pub mod manager;
pub mod vault;

pub use aws::AwsSecretsManager;
pub use kubernetes::KubernetesSecretManager;
pub use manager::{Secret, SecretManager};
pub use vault::VaultSecretManager;

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

    pub async fn inject_secrets(
        &self,
        command: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.manager.inject_secrets(command).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_aws_secrets_manager() {
        let mgr = AwsSecretsManager::new("us-west-2".to_string());
        let service = SecretsService::new(Box::new(mgr));
        let secret = service.get_secret("api_key").await.unwrap();
        assert_eq!(secret, "aws_secret_for_api_key");
        let injected = service.inject_secrets("echo $KEY").await.unwrap();
        assert_eq!(injected, "echo $KEY");
    }

    #[tokio::test]
    async fn test_vault_secrets_manager() {
        let mgr = VaultSecretManager::new("https://vault.local:8200".to_string(), "token123".to_string());
        let service = SecretsService::new(Box::new(mgr));
        let secret = service.get_secret("db_pass").await.unwrap();
        assert_eq!(secret, "secret_value_for_db_pass");
        let injected = service.inject_secrets("forge build").await.unwrap();
        assert_eq!(injected, "forge build");
    }

    #[tokio::test]
    async fn test_kubernetes_secrets_manager() {
        let mgr = KubernetesSecretManager::new("prod".to_string());
        let service = SecretsService::new(Box::new(mgr));
        let secret = service.get_secret("tls_cert").await.unwrap();
        assert_eq!(secret, "k8s_secret_for_tls_cert");
        let injected = service.inject_secrets("cargo test").await.unwrap();
        assert_eq!(injected, "cargo test");
    }

    #[test]
    fn test_secret_debug_redaction() {
        let s = Secret {
            key: "API_KEY".to_string(),
            value: "super_secret_value".to_string(),
            version: Some("v1".to_string()),
        };
        let formatted = format!("{:?}", s);
        assert!(formatted.contains("[REDACTED]"));
        assert!(!formatted.contains("super_secret_value"));
        assert!(formatted.contains("API_KEY"));
        assert!(formatted.contains("v1"));
    }
}
