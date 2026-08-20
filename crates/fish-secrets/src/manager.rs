// Secret manager trait

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Secret {
    pub key: String,
    pub value: String,
    pub version: Option<String>,
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secret")
            .field("key", &self.key)
            .field("value", &"[REDACTED]")
            .field("version", &self.version)
            .finish()
    }
}

#[async_trait::async_trait]
pub trait SecretManager: Send + Sync {
    async fn get_secret(&self, key: &str) -> Result<String, anyhow::Error>;
    async fn inject_secrets(&self, command: &str) -> Result<String, anyhow::Error>;
}
