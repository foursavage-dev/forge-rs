// Metrics aggregator

use crate::metrics::CacheMetrics;
use std::path::Path;

#[derive(Clone)]
pub struct MetricsAggregator;

impl MetricsAggregator {
    pub fn new() -> Self {
        Self
    }

    pub async fn collect(&self, _project_path: &Path) -> Result<CacheMetrics, Box<dyn std::error::Error>> {
        Ok(CacheMetrics {
            hit_rate: 0.0,
            total_hits: 0,
            total_misses: 0,
            total_requests: 0,
            cache_size_bytes: 0,
            timestamp: chrono::Utc::now(),
        })
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}
