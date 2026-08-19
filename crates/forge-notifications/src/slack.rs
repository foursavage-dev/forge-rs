// Slack notifier

use crate::notifier::{Notification, NotificationConfig, Notifier};

pub struct SlackNotifier {
    config: NotificationConfig,
}

impl SlackNotifier {
    pub fn new(config: NotificationConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Notifier for SlackNotifier {
    async fn send(&self, notification: Notification) -> Result<(), anyhow::Error> {
        if let Some(webhook_url) = &self.config.webhook_url {
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "text": notification.title,
                "attachments": [{
                    "text": notification.message,
                    "color": match notification.status {
                        crate::notifier::BuildStatus::Success => "good",
                        crate::notifier::BuildStatus::Failure => "danger",
                        _ => "warning",
                    }
                }]
            });
            client.post(webhook_url).json(&payload).send().await?;
        }
        Ok(())
    }
}
