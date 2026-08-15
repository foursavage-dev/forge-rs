// Metrics data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub hit_rate: f64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_requests: u64,
    pub cache_size_bytes: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetrics {
    pub build_duration_secs: f64,
    pub cache_saved_time_secs: f64,
    pub packages_built: u32,
    pub packages_cached: u32,
    pub timestamp: DateTime<Utc>,
}
