// HashiCorp Vault integration

use crate::manager::SecretManager;

pub struct VaultSecretManager {
    address: String,
    #[allow(dead_code)] // part of the integration config; never logged or printed
    token: String,
}

impl VaultSecretManager {
    pub fn new(address: String, token: String) -> Self {
        Self { address, token }
    }
}

#[async_trait::async_trait]
impl SecretManager for VaultSecretManager {
    async fn get_secret(&self, key: &str) -> Result<String, anyhow::Error> {
        // The Vault API integration is not implemented yet. Failing loudly
        // prevents callers from silently receiving a fabricated value.
        Err(anyhow::anyhow!(
            "HashiCorp Vault integration is not implemented yet (requested `{key}` from `{}`)",
            self.address
        ))
    }

    async fn inject_secrets(&self, command: &str) -> Result<String, anyhow::Error> {
        Err(anyhow::anyhow!(
            "HashiCorp Vault secret injection is not implemented yet (command `{command}`)"
        ))
    }
}
