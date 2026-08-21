// Analytics dashboard

#[derive(Clone)]
pub struct AnalyticsDashboard {
    config: DashboardConfig,
}

#[derive(Debug, Clone)]
pub struct DashboardConfig {
    pub port: u16,
    pub refresh_interval_secs: u64,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            refresh_interval_secs: 5,
        }
    }
}

impl AnalyticsDashboard {
    pub fn new(config: DashboardConfig) -> Self {
        Self { config }
    }

    /// The web analytics dashboard is implemented by the `fish-dashboard`
    /// crate (`DashboardServer`). This standalone stub used to return `Ok(())`
    /// without starting anything; it now fails loudly so callers do not
    /// believe a dashboard is serving on `port` when nothing is listening.
    pub async fn start(&self) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!(
            "the analytics web dashboard is not implemented here; use `fish-dashboard::DashboardServer` (port {}) instead",
            self.config.port
        ))
    }
}
