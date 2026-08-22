// Metrics data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Aggregated team build metrics for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamBuildStats {
    pub total_builds: u64,
    pub avg_build_duration_secs: f64,
    pub p50_duration_secs: f64,
    pub p95_duration_secs: f64,
    pub cache_hit_rate_avg: f64,
    pub total_time_saved_secs: f64,
    pub builds_per_day: HashMap<String, u64>,
    pub top_slowest_packages: Vec<PackageTiming>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTiming {
    pub package_name: String,
    pub avg_duration_secs: f64,
    pub build_count: u64,
}

/// Team velocity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamVelocityMetrics {
    pub active_developers: u32,
    pub builds_last_24h: u64,
    pub builds_last_7d: u64,
    pub cache_efficiency_trend: Vec<CacheEfficiencyPoint>,
    pub velocity_score: f64, // 0-100
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEfficiencyPoint {
    pub date: String,
    pub hit_rate: f64,
    pub time_saved_secs: f64,
}

/// Cloud cost calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCostMetrics {
    pub compute_cost_without_cache_usd: f64,
    pub compute_cost_with_cache_usd: f64,
    pub storage_cost_usd: f64,
    pub total_savings_usd: f64,
    pub savings_percentage: f64,
    pub breakdown: CostBreakdown,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub cpu_hours_saved: f64,
    pub cache_storage_gb: f64,
    pub egress_gb: f64,
    pub remote_cache_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostConfig {
    pub cpu_cost_per_hour_usd: f64,
    pub storage_cost_per_gb_month_usd: f64,
    pub egress_cost_per_gb_usd: f64,
    pub remote_cache_cost_per_10k_requests_usd: f64,
}

impl Default for CostConfig {
    fn default() -> Self {
        Self {
            cpu_cost_per_hour_usd: 0.10,               // $0.10 per vCPU-hour
            storage_cost_per_gb_month_usd: 0.023,       // S3 standard
            egress_cost_per_gb_usd: 0.09,
            remote_cache_cost_per_10k_requests_usd: 0.01,
        }
    }
}

impl CacheMetrics {
    pub fn calculate_savings(&self, avg_miss_duration_secs: f64) -> f64 {
        self.total_hits as f64 * avg_miss_duration_secs
    }
}
