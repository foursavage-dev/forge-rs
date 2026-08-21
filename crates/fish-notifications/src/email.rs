use crate::notifier::{Notification, NotificationConfig, Notifier};

pub struct EmailNotifier {
    config: NotificationConfig,
}

impl EmailNotifier {
    pub fn new(config: NotificationConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Notifier for EmailNotifier {
    async fn send(&self, notification: Notification) -> Result<(), anyhow::Error> {
        if let Some(webhook_url) = &self.config.webhook_url {
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "type": "email_notification",
                "subject": notification.title,
                "body": notification.message,
                "status": format!("{:?}", notification.status),
                "timestamp": notification.timestamp
            });
            // Propagate delivery failures like the other notifiers instead of
            // swallowing them and reporting a successful send.
            client.post(webhook_url).json(&payload).send().await?;
        }
        Ok(())
    }
}
