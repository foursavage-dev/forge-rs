// Fish Analytics - Build Cache Analytics Dashboard
// Provides real-time analytics for cache performance

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod aggregator;
pub mod dashboard;
pub mod metrics;
pub mod otel;

pub use aggregator::MetricsAggregator;
pub use dashboard::{AnalyticsDashboard, CostCalculator, DashboardConfig, DashboardState};
pub use metrics::{
    BuildMetrics, CacheMetrics, CloudCostMetrics, CostBreakdown, CostConfig, PackageTiming,
    TeamBuildStats, TeamVelocityMetrics,
};
pub use otel::{
    ActiveSpanBuilder, AttributeValue, OtelExporter, OtelSpan, OtelTracer, SpanEvent, SpanKind,
    SpanStatus, StatusCode,
};

use std::path::Path;

/// Main analytics service - full implementation
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

    pub fn with_cost_config(cost_config: CostConfig) -> Self {
        Self {
            aggregator: MetricsAggregator::with_cost_config(cost_config),
        }
    }

    pub async fn collect_metrics(
        &self,
        project_path: &Path,
    ) -> Result<CacheMetrics, anyhow::Error> {
        self.aggregator.collect(project_path).await
    }

    pub async fn collect_team_stats(
        &self,
        builds: &[BuildMetrics],
    ) -> Result<TeamBuildStats, anyhow::Error> {
        self.aggregator.collect_team_stats(builds).await
    }

    pub fn calculate_cost(
        &self,
        cache_metrics: &CacheMetrics,
        team_stats: &TeamBuildStats,
    ) -> CloudCostMetrics {
        self.aggregator
            .calculate_cost_metrics(cache_metrics, team_stats)
    }

    pub fn calculate_velocity(
        &self,
        team_stats: &TeamBuildStats,
        builds_last_24h: u64,
        builds_last_7d: u64,
        active_devs: u32,
    ) -> TeamVelocityMetrics {
        self.aggregator
            .calculate_velocity(team_stats, builds_last_24h, builds_last_7d, active_devs)
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
        // Metrics may be zero if no cache, but should not error
        assert!(metrics.hit_rate >= 0.0);
        assert!(metrics.hit_rate <= 1.0);
    }

    #[tokio::test]
    async fn test_dashboard_config_and_metrics() {
        let config = DashboardConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.refresh_interval_secs, 5);

        let dashboard = AnalyticsDashboard::new(config);
        let metrics = dashboard.get_metrics().await.unwrap();
        assert!(metrics.cache.is_some());
        assert!(metrics.team_stats.is_some());
        assert!(metrics.cost.is_some());
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

    #[tokio::test]
    async fn test_team_stats_and_cost() {
        let service = AnalyticsService::new();
        let builds = vec![
            BuildMetrics {
                build_duration_secs: 45.0,
                cache_saved_time_secs: 120.0,
                packages_built: 5,
                packages_cached: 20,
                timestamp: chrono::Utc::now(),
            },
            BuildMetrics {
                build_duration_secs: 30.0,
                cache_saved_time_secs: 100.0,
                packages_built: 3,
                packages_cached: 22,
                timestamp: chrono::Utc::now(),
            },
        ];

        let team_stats = service.collect_team_stats(&builds).await.unwrap();
        assert_eq!(team_stats.total_builds, 2);
        assert!(team_stats.avg_build_duration_secs > 0.0);

        let cache_metrics = CacheMetrics {
            hit_rate: 0.85,
            total_hits: 100,
            total_misses: 20,
            total_requests: 120,
            cache_size_bytes: 1024 * 1024 * 100,
            timestamp: chrono::Utc::now(),
        };

        let cost = service.calculate_cost(&cache_metrics, &team_stats);
        assert!(cost.total_savings_usd >= 0.0);

        let velocity = service.calculate_velocity(&team_stats, 10, 50, 5);
        assert!(velocity.velocity_score > 0.0);
        assert!(velocity.velocity_score <= 100.0);
    }

    #[test]
    fn test_cost_calculator() {
        let calc = CostCalculator::new(CostConfig::default());
        let cost = calc.estimate_monthly_savings(100, 120.0, 0.8, 4.0);
        assert!(cost.total_savings_usd >= 0.0);
        assert!(cost.breakdown.cpu_hours_saved > 0.0);
    }
}
