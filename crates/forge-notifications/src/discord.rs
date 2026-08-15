// Discord notifier

use crate::notifier::{Notifier, Notification, NotificationConfig};

pub struct DiscordNotifier {
    config: NotificationConfig,
}

impl DiscordNotifier {
    pub fn new(config: NotificationConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Notifier for DiscordNotifier {
    async fn send(&self, notification: Notification) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(webhook_url) = &self.config.webhook_url {
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "content": notification.title,
                "embeds": [{
                    "description": notification.message,
                    "color": match notification.status {
                        crate::notifier::BuildStatus::Success => 65280,
                        crate::notifier::BuildStatus::Failure => 16711680,
                        _ => 16776960,
                    }
                }]
            });
            client.post(webhook_url).json(&payload).send().await?;
        }
        Ok(())
    }
}
