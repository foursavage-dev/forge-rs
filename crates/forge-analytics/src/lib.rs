// Forge Analytics - Build Cache Analytics Dashboard
// Provides real-time analytics for cache performance

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod aggregator;
pub mod dashboard;
pub mod metrics;

pub use aggregator::MetricsAggregator;
pub use dashboard::{AnalyticsDashboard, DashboardConfig};
pub use metrics::{BuildMetrics, CacheMetrics};

use std::path::Path;

/// Main analytics service
#[derive(Clone)]
pub struct AnalyticsService {
    aggregator: MetricsAggregator,
}

impl AnalyticsService {
    pub fn new() -> Self {
        Self {
            aggregator: MetricsAggregator::new(),
        }
    }

    pub async fn collect_metrics(
        &self,
        project_path: &Path,
    ) -> Result<CacheMetrics, anyhow::Error> {
        self.aggregator.collect(project_path).await
    }
}

impl Default for AnalyticsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_analytics_service_collect_metrics() {
        let service = AnalyticsService::new();
        let path = PathBuf::from(".");
        let metrics = service.collect_metrics(&path).await.unwrap();
        assert_eq!(metrics.total_hits, 0);
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.hit_rate, 0.0);
    }

    #[tokio::test]
    async fn test_dashboard_config_and_start() {
        let config = DashboardConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.refresh_interval_secs, 5);

        let dashboard = AnalyticsDashboard::new(config);
        let result = dashboard.start().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_metrics_serialization() {
        let metrics = BuildMetrics {
            build_duration_secs: 12.5,
            cache_saved_time_secs: 45.2,
            packages_built: 4,
            packages_cached: 10,
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("12.5"));
        assert!(json.contains("packages_cached"));
    }
}
