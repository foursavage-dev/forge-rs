// Notifier trait and types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub status: BuildStatus,
    pub build_id: String,
    pub timestamp: DateTime<Utc>,
    pub logs_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildStatus {
    Success,
    Failure,
    Flaky,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub webhook_url: Option<String>,
    pub channel: Option<String>,
    pub username: Option<String>,
}

#[async_trait::async_trait]
pub trait Notifier: Send + Sync {
    async fn send(&self, notification: Notification) -> Result<(), anyhow::Error>;
}
