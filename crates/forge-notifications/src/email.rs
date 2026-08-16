// Email notifier

use crate::notifier::{Notification, NotificationConfig, Notifier};

pub struct EmailNotifier {
    #[allow(dead_code)]
    config: NotificationConfig,
}

impl EmailNotifier {
    pub fn new(config: NotificationConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Notifier for EmailNotifier {
    async fn send(&self, _notification: Notification) -> Result<(), Box<dyn std::error::Error>> {
        // Placeholder for email sending
        Ok(())
    }
}
