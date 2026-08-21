// AWS Secrets Manager integration

use crate::manager::SecretManager;

pub struct AwsSecretsManager {
    region: String,
}

impl AwsSecretsManager {
    pub fn new(region: String) -> Self {
        Self { region }
    }
}

#[async_trait::async_trait]
impl SecretManager for AwsSecretsManager {
    async fn get_secret(&self, key: &str) -> Result<String, anyhow::Error> {
        // The AWS SDK integration is not implemented yet. Failing loudly
        // prevents callers from silently receiving a fabricated value.
        Err(anyhow::anyhow!(
            "AWS Secrets Manager integration is not implemented yet (requested `{key}` in region `{}`)",
            self.region
        ))
    }

    async fn inject_secrets(&self, command: &str) -> Result<String, anyhow::Error> {
        Err(anyhow::anyhow!(
            "AWS Secrets Manager secret injection is not implemented yet (command `{command}`)"
        ))
    }
}
