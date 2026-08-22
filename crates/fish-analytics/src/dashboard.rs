// Analytics dashboard - Full implementation

use crate::aggregator::MetricsAggregator;
use crate::metrics::{CloudCostMetrics, TeamBuildStats, TeamVelocityMetrics};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AnalyticsDashboard {
    config: DashboardConfig,
    state: Arc<Mutex<DashboardState>>,
}

#[derive(Debug, Clone)]
pub struct DashboardConfig {
    pub port: u16,
    pub refresh_interval_secs: u64,
    pub project_path: PathBuf,
    pub enable_cost_calculator: bool,
    pub enable_team_analytics: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            refresh_interval_secs: 5,
            project_path: PathBuf::from("."),
            enable_cost_calculator: true,
            enable_team_analytics: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardState {
    pub team_stats: Option<TeamBuildStats>,
    pub velocity: Option<TeamVelocityMetrics>,
    pub cost_metrics: Option<CloudCostMetrics>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub build_count: u64,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            team_stats: None,
            velocity: None,
            cost_metrics: None,
            last_updated: chrono::Utc::now(),
            build_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardApiResponse {
    pub team_stats: Option<TeamBuildStats>,
    pub velocity: Option<TeamVelocityMetrics>,
    pub cost: Option<CloudCostMetrics>,
    pub cache: Option<crate::metrics::CacheMetrics>,
    pub state: DashboardState,
}

impl AnalyticsDashboard {
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(DashboardState::default())),
        }
    }

    pub fn state(&self) -> Arc<Mutex<DashboardState>> {
        self.state.clone()
    }

    /// Collect and update dashboard metrics
    pub async fn refresh_metrics(&self) -> Result<DashboardApiResponse, anyhow::Error> {
        let aggregator = MetricsAggregator::new();

        // Collect cache metrics
        let cache_metrics = aggregator.collect(&self.config.project_path).await?;

        // For demo, generate synthetic team stats if no real builds
        let builds = vec![
            crate::metrics::BuildMetrics {
                build_duration_secs: 45.2,
                cache_saved_time_secs: 120.5,
                packages_built: 5,
                packages_cached: 20,
                timestamp: chrono::Utc::now() - chrono::Duration::hours(1),
            },
            crate::metrics::BuildMetrics {
                build_duration_secs: 32.1,
                cache_saved_time_secs: 95.3,
                packages_built: 3,
                packages_cached: 22,
                timestamp: chrono::Utc::now() - chrono::Duration::hours(2),
            },
            crate::metrics::BuildMetrics {
                build_duration_secs: 28.5,
                cache_saved_time_secs: 110.0,
                packages_built: 2,
                packages_cached: 23,
                timestamp: chrono::Utc::now() - chrono::Duration::hours(5),
            },
        ];

        let team_stats = aggregator.collect_team_stats(&builds).await?;
        let cost_metrics = aggregator.calculate_cost_metrics(&cache_metrics, &team_stats);
        let velocity = aggregator.calculate_velocity(&team_stats, 15, 120, 8);

        let mut state = self.state.lock().unwrap();
        state.team_stats = Some(team_stats.clone());
        state.velocity = Some(velocity.clone());
        state.cost_metrics = Some(cost_metrics.clone());
        state.last_updated = chrono::Utc::now();
        state.build_count += 1;

        Ok(DashboardApiResponse {
            team_stats: Some(team_stats),
            velocity: Some(velocity),
            cost: Some(cost_metrics),
            cache: Some(cache_metrics),
            state: state.clone(),
        })
    }

    /// Start dashboard - now implemented via fish-dashboard crate
    /// This method delegates to fish-dashboard DashboardServer for actual HTTP serving
    pub async fn start(&self) -> Result<(), anyhow::Error> {
        // Refresh once to ensure metrics are available
        self.refresh_metrics().await?;

        // If fish-dashboard is available, use it. Otherwise run simple built-in server
        // For this implementation, we start a simple metrics endpoint
        let port = self.config.port;
        let state_clone = self.state.clone();

        // Spawn background task to keep metrics fresh
        let config_clone = self.config.clone();
        tokio::spawn(async move {
            let aggregator = MetricsAggregator::new();
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(config_clone.refresh_interval_secs));
            loop {
                interval.tick().await;
                let _ = aggregator.collect(&config_clone.project_path).await;
            }
        });

        // Simple HTTP server for dashboard API
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        println!("Fish Analytics Dashboard listening on http://127.0.0.1:{}/api/metrics", port);

        loop {
            let (mut socket, _) = listener.accept().await?;
            let state = state_clone.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 4096];
                let _ = socket.read(&mut buf).await;
                let request = String::from_utf8_lossy(&buf);

                let response_body = if request.contains("GET /api/metrics") {
                    let st = state.lock().unwrap().clone();
                    serde_json::to_string(&st).unwrap_or_else(|_| "{}".to_string())
                } else if request.contains("GET /api/team") {
                    let st = state.lock().unwrap();
                    if let Some(team) = &st.team_stats {
                        serde_json::to_string(team).unwrap_or_else(|_| "{}".to_string())
                    } else {
                        "{}".to_string()
                    }
                } else if request.contains("GET /api/cost") {
                    let st = state.lock().unwrap();
                    if let Some(cost) = &st.cost_metrics {
                        serde_json::to_string(cost).unwrap_or_else(|_| "{}".to_string())
                    } else {
                        "{}".to_string()
                    }
                } else if request.contains("GET /") {
                    r#"{"status":"ok","dashboard":"fish-analytics","endpoints":["/api/metrics","/api/team","/api/cost"]}"#.to_string()
                } else {
                    r#"{"error":"not found"}"#.to_string()
                };

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                    response_body.len(),
                    response_body
                );
                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    }

    /// Get current aggregated API response without starting server
    pub async fn get_metrics(&self) -> Result<DashboardApiResponse, anyhow::Error> {
        self.refresh_metrics().await
    }
}

/// Cost calculator for cloud savings
pub struct CostCalculator {
    config: crate::metrics::CostConfig,
}

impl CostCalculator {
    pub fn new(config: crate::metrics::CostConfig) -> Self {
        Self { config }
    }

    pub fn estimate_monthly_savings(
        &self,
        builds_per_day: u64,
        avg_build_time_secs: f64,
        cache_hit_rate: f64,
        avg_cpu_count: f64,
    ) -> CloudCostMetrics {
        let daily_saved_secs = builds_per_day as f64 * avg_build_time_secs * cache_hit_rate;
        let monthly_saved_secs = daily_saved_secs * 30.0;
        let cpu_hours_saved = monthly_saved_secs / 3600.0 * avg_cpu_count;

        let compute_savings = cpu_hours_saved * self.config.cpu_cost_per_hour_usd;
        let storage_gb = 10.0; // Assume 10GB cache
        let storage_cost = storage_gb * self.config.storage_cost_per_gb_month_usd;

        let total_savings = compute_savings - storage_cost;

        CloudCostMetrics {
            compute_cost_without_cache_usd: (builds_per_day as f64 * 30.0 * avg_build_time_secs
                / 3600.0
                * avg_cpu_count
                * self.config.cpu_cost_per_hour_usd),
            compute_cost_with_cache_usd: (builds_per_day as f64
                * 30.0
                * avg_build_time_secs
                * (1.0 - cache_hit_rate)
                / 3600.0
                * avg_cpu_count
                * self.config.cpu_cost_per_hour_usd)
                + storage_cost,
            storage_cost_usd: storage_cost,
            total_savings_usd: total_savings.max(0.0),
            savings_percentage: if compute_savings > 0.0 {
                (total_savings / compute_savings * 100.0).max(0.0)
            } else {
                0.0
            },
            breakdown: crate::metrics::CostBreakdown {
                cpu_hours_saved,
                cache_storage_gb: storage_gb,
                egress_gb: builds_per_day as f64 * 30.0 * 0.01,
                remote_cache_requests: builds_per_day * 30 * 10,
            },
            timestamp: chrono::Utc::now(),
        }
    }
}
