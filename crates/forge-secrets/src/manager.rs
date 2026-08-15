// Secret manager trait

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub key: String,
    pub value: String,
    pub version: Option<String>,
}

#[async_trait::async_trait]
pub trait SecretManager: Send + Sync {
    async fn get_secret(&self, key: &str) -> Result<String, Box<dyn std::error::Error>>;
    async fn inject_secrets(&self, command: &str) -> Result<String, Box<dyn std::error::Error>>;
}
