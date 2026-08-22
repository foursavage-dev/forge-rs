#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::metrics::{BuildMetrics, BuildStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAnalytics {
    pub total_builds: u64,
    pub successful_builds: u64,
    pub failed_builds: u64,
    pub avg_build_duration_ms: f64,
    pub p50_duration_ms: f64,
    pub p95_duration_ms: f64,
    pub cache_hit_rate_avg: f64,
    pub total_time_saved_ms: u64,
    pub builds_per_day: HashMap<String, u64>,
    pub builds_per_developer: HashMap<String, u64>,
    pub slowest_packages: Vec<PackageStats>,
    pub velocity_score: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageStats {
    pub package_name: String,
    pub avg_duration_ms: f64,
    pub build_count: u64,
    pub cache_hit_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityMetrics {
    pub active_developers: u32,
    pub builds_last_24h: u64,
    pub builds_last_7d: u64,
    pub avg_builds_per_dev_per_day: f64,
    pub cache_efficiency_trend: Vec<EfficiencyPoint>,
    pub velocity_trend: Vec<VelocityPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyPoint {
    pub date: String,
    pub hit_rate: f64,
    pub time_saved_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityPoint {
    pub date: String,
    pub build_count: u64,
    pub avg_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetrics {
    pub compute_cost_without_cache_usd: f64,
    pub compute_cost_with_cache_usd: f64,
    pub storage_cost_usd: f64,
    pub total_savings_usd: f64,
    pub savings_percentage: f64,
    pub cpu_hours_saved: f64,
    pub breakdown: CostBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub cpu_cost: f64,
    pub storage_gb: f64,
    pub egress_gb: f64,
    pub cache_requests: u64,
}

impl TeamAnalytics {
    pub fn from_builds(builds: &[BuildMetrics]) -> Self {
        if builds.is_empty() {
            return Self {
                total_builds: 0,
                successful_builds: 0,
                failed_builds: 0,
                avg_build_duration_ms: 0.0,
                p50_duration_ms: 0.0,
                p95_duration_ms: 0.0,
                cache_hit_rate_avg: 0.0,
                total_time_saved_ms: 0,
                builds_per_day: HashMap::new(),
                builds_per_developer: HashMap::new(),
                slowest_packages: Vec::new(),
                velocity_score: 0.0,
                timestamp: Utc::now(),
            };
        }

        let total_builds = builds.len() as u64;
        let successful_builds = builds
            .iter()
            .filter(|b| b.status == BuildStatus::Success)
            .count() as u64;
        let failed_builds = builds
            .iter()
            .filter(|b| b.status == BuildStatus::Failed)
            .count() as u64;

        let mut durations: Vec<u64> = builds
            .iter()
            .filter_map(|b| b.duration_ms)
            .collect();
        durations.sort();

        let avg_duration = if !durations.is_empty() {
            durations.iter().sum::<u64>() as f64 / durations.len() as f64
        } else {
            0.0
        };

        let p50 = Self::percentile(&durations, 0.5) as f64;
        let p95 = Self::percentile(&durations, 0.95) as f64;

        let cache_hit_avg = if !builds.is_empty() {
            builds.iter().map(|b| b.cache_stats.hit_rate).sum::<f64>() / builds.len() as f64
        } else {
            0.0
        };

        let total_saved: u64 = builds.iter().map(|b| b.cache_stats.bytes_saved).sum();

        // Builds per day
        let mut per_day: HashMap<String, u64> = HashMap::new();
        for build in builds {
            let day = build.start_time.format("%Y-%m-%d").to_string();
            *per_day.entry(day).or_insert(0) += 1;
        }

        // Velocity score: based on cache hit, build time, success rate
        let success_rate = if total_builds > 0 {
            successful_builds as f64 / total_builds as f64
        } else {
            0.0
        };

        let mut velocity_score = 50.0;
        velocity_score += cache_hit_avg * 20.0;
        velocity_score += success_rate * 20.0;
        if avg_duration < 60_000.0 {
            velocity_score += 10.0;
        } else if avg_duration < 300_000.0 {
            velocity_score += 5.0;
        }
        velocity_score = velocity_score.min(100.0);

        // Slowest packages from tasks
        let mut package_times: HashMap<String, Vec<u64>> = HashMap::new();
        for build in builds {
            for task in &build.tasks {
                if let Some(duration) = task.duration_ms {
                    package_times
                        .entry(task.description.clone())
                        .or_default()
                        .push(duration);
                }
            }
        }

        let mut slowest: Vec<PackageStats> = package_times
            .into_iter()
            .map(|(name, times)| {
                let avg = times.iter().sum::<u64>() as f64 / times.len() as f64;
                PackageStats {
                    package_name: name,
                    avg_duration_ms: avg,
                    build_count: times.len() as u64,
                    cache_hit_rate: 0.0, // Would need task cache data
                }
            })
            .collect();

        slowest.sort_by(|a, b| b.avg_duration_ms.partial_cmp(&a.avg_duration_ms).unwrap());
        slowest.truncate(10);

        Self {
            total_builds,
            successful_builds,
            failed_builds,
            avg_build_duration_ms: avg_duration,
            p50_duration_ms: p50,
            p95_duration_ms: p95,
            cache_hit_rate_avg: cache_hit_avg,
            total_time_saved_ms: total_saved,
            builds_per_day: per_day,
            builds_per_developer: HashMap::new(),
            slowest_packages: slowest,
            velocity_score,
            timestamp: Utc::now(),
        }
    }

    fn percentile(sorted: &[u64], p: f64) -> u64 {
        if sorted.is_empty() {
            return 0;
        }
        let idx = (p * sorted.len() as f64).ceil() as usize - 1;
        let idx = idx.min(sorted.len() - 1);
        sorted[idx]
    }

    pub fn calculate_cost(&self, cost_per_cpu_hour: f64) -> CostMetrics {
        let total_duration_hours = self.avg_build_duration_ms * self.total_builds as f64 / 1000.0 / 3600.0;
        let saved_hours = self.total_time_saved_ms as f64 / 1000.0 / 3600.0;

        let cost_without_cache = (total_duration_hours + saved_hours) * cost_per_cpu_hour;
        let cost_with_cache = total_duration_hours * cost_per_cpu_hour + 5.0; // $5 storage
        let savings = cost_without_cache - cost_with_cache;
        let savings_pct = if cost_without_cache > 0.0 {
            savings / cost_without_cache * 100.0
        } else {
            0.0
        };

        CostMetrics {
            compute_cost_without_cache_usd: cost_without_cache,
            compute_cost_with_cache_usd: cost_with_cache,
            storage_cost_usd: 5.0,
            total_savings_usd: savings.max(0.0),
            savings_percentage: savings_pct.max(0.0),
            cpu_hours_saved: saved_hours,
            breakdown: CostBreakdown {
                cpu_cost: cost_with_cache,
                storage_gb: 10.0,
                egress_gb: self.total_builds as f64 * 0.01,
                cache_requests: self.total_builds * 10,
            },
        }
    }
}

impl VelocityMetrics {
    pub fn from_analytics(analytics: &TeamAnalytics, active_devs: u32) -> Self {
        let builds_last_24h = analytics
            .builds_per_day
            .iter()
            .filter(|(date, _)| {
                // Simple check for recent dates
                date.as_str() >= Utc::now().format("%Y-%m-%d").to_string().as_str()
            })
            .map(|(_, count)| count)
            .sum();

        let builds_last_7d = analytics.builds_per_day.values().sum();

        let avg_per_dev = if active_devs > 0 {
            builds_last_7d as f64 / active_devs as f64 / 7.0
        } else {
            0.0
        };

        let mut efficiency_trend: Vec<EfficiencyPoint> = analytics
            .builds_per_day
            .iter()
            .map(|(date, _)| EfficiencyPoint {
                date: date.clone(),
                hit_rate: analytics.cache_hit_rate_avg,
                time_saved_ms: analytics.total_time_saved_ms / analytics.total_builds.max(1),
            })
            .collect();
        efficiency_trend.sort_by(|a, b| a.date.cmp(&b.date));

        let mut velocity_trend: Vec<VelocityPoint> = analytics
            .builds_per_day
            .iter()
            .map(|(date, count)| VelocityPoint {
                date: date.clone(),
                build_count: *count,
                avg_duration_ms: analytics.avg_build_duration_ms,
            })
            .collect();
        velocity_trend.sort_by(|a, b| a.date.cmp(&b.date));

        Self {
            active_developers: active_devs,
            builds_last_24h,
            builds_last_7d,
            avg_builds_per_dev_per_day: avg_per_dev,
            cache_efficiency_trend: efficiency_trend,
            velocity_trend,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{BuildMetrics, CacheStats};

    #[test]
    fn test_team_analytics_from_builds() {
        let mut builds = Vec::new();
        for i in 0..10 {
            let mut build = BuildMetrics::new(
                format!("build-{}", i),
                "test-project".to_string(),
                "rust".to_string(),
            );
            build.cache_stats = CacheStats {
                hits: 80,
                misses: 20,
                hit_rate: 0.8,
                bytes_saved: 1024 * 1024,
            };
            build.duration_ms = Some(5000 + i * 100);
            build.status = if i % 10 == 0 {
                BuildStatus::Failed
            } else {
                BuildStatus::Success
            };
            builds.push(build);
        }

        let analytics = TeamAnalytics::from_builds(&builds);
        assert_eq!(analytics.total_builds, 10);
        assert!(analytics.avg_build_duration_ms > 0.0);
        assert!(analytics.cache_hit_rate_avg > 0.7);
        assert!(analytics.velocity_score > 0.0);

        let cost = analytics.calculate_cost(0.10);
        assert!(cost.total_savings_usd >= 0.0);
    }

    #[test]
    fn test_velocity_metrics() {
        let builds = vec![BuildMetrics::new(
            "build-1".to_string(),
            "test".to_string(),
            "rust".to_string(),
        )];

        let analytics = TeamAnalytics::from_builds(&builds);
        let velocity = VelocityMetrics::from_analytics(&analytics, 5);
        assert_eq!(velocity.active_developers, 5);
    }
}
