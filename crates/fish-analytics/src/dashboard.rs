// Analytics dashboard

#[derive(Clone)]
pub struct AnalyticsDashboard {
    #[allow(dead_code)]
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

    pub async fn start(&self) -> Result<(), anyhow::Error> {
        // Would start web server here
        Ok(())
    }
}
