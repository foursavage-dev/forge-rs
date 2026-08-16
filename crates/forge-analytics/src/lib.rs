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
    ) -> Result<CacheMetrics, Box<dyn std::error::Error>> {
        self.aggregator.collect(project_path).await
    }
}

impl Default for AnalyticsService {
    fn default() -> Self {
        Self::new()
    }
}
