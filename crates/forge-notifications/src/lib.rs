// Forge Notifications - Build Notification System
// Slack/Discord/Email notifications for build status

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod discord;
pub mod email;
pub mod notifier;
pub mod slack;

pub use discord::DiscordNotifier;
pub use email::EmailNotifier;
pub use notifier::{Notification, NotificationConfig, Notifier};
pub use slack::SlackNotifier;

/// Main notification service
pub struct NotificationService {
    notifiers: Vec<Box<dyn Notifier>>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            notifiers: Vec::new(),
        }
    }

    pub fn add_notifier(&mut self, notifier: Box<dyn Notifier>) {
        self.notifiers.push(notifier);
    }

    pub async fn send(&self, notification: Notification) -> Result<(), Box<dyn std::error::Error>> {
        for notifier in &self.notifiers {
            notifier.send(notification.clone()).await?;
        }
        Ok(())
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}
