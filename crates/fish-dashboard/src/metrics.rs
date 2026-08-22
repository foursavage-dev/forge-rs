#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetrics {
    pub build_id: String,
    pub project_name: String,
    pub backend: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub status: BuildStatus,
    pub tasks: Vec<TaskMetrics>,
    pub cache_stats: CacheStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildStatus {
    Running,
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub task_id: String,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub status: BuildStatus,
    pub cache_hit: bool,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub bytes_saved: u64,
}

impl BuildMetrics {
    pub fn new(build_id: String, project_name: String, backend: String) -> Self {
        Self {
            build_id,
            project_name,
            backend,
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            status: BuildStatus::Running,
            tasks: Vec::new(),
            cache_stats: CacheStats {
                hits: 0,
                misses: 0,
                hit_rate: 0.0,
                bytes_saved: 0,
            },
        }
    }

    pub fn complete(&mut self, status: BuildStatus) {
        self.end_time = Some(Utc::now());
        self.duration_ms = Some(
            self.end_time
                .unwrap()
                .signed_duration_since(self.start_time)
                .num_milliseconds() as u64,
        );
        self.status = status;

        // Calculate cache hit rate
        let total = self.cache_stats.hits + self.cache_stats.misses;
        if total > 0 {
            self.cache_stats.hit_rate = self.cache_stats.hits as f64 / total as f64;
        }
    }

    pub fn add_task(&mut self, task: TaskMetrics) {
        self.tasks.push(task);
    }

    pub fn update_cache_stats(&mut self, hits: u64, misses: u64, bytes_saved: u64) {
        self.cache_stats.hits = hits;
        self.cache_stats.misses = misses;
        self.cache_stats.bytes_saved = bytes_saved;
    }
}

impl TaskMetrics {
    pub fn new(task_id: String, description: String) -> Self {
        Self {
            task_id,
            description,
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            status: BuildStatus::Running,
            cache_hit: false,
            dependencies: Vec::new(),
        }
    }

    pub fn complete(&mut self, status: BuildStatus, cache_hit: bool) {
        self.end_time = Some(Utc::now());
        self.duration_ms = Some(
            self.end_time
                .unwrap()
                .signed_duration_since(self.start_time)
                .num_milliseconds() as u64,
        );
        self.status = status;
        self.cache_hit = cache_hit;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStore {
    builds: HashMap<String, BuildMetrics>,
    recent_builds: Vec<String>,
    max_recent: usize,
}

impl MetricsStore {
    pub fn new(max_recent: usize) -> Self {
        Self {
            builds: HashMap::new(),
            recent_builds: Vec::new(),
            max_recent,
        }
    }

    pub fn add_build(&mut self, metrics: BuildMetrics) {
        let build_id = metrics.build_id.clone();
        self.builds.insert(build_id.clone(), metrics);

        // Update recent builds
        if !self.recent_builds.contains(&build_id) {
            self.recent_builds.push(build_id.clone());
            if self.recent_builds.len() > self.max_recent {
                self.recent_builds.remove(0);
            }
        }
    }

    pub fn get_build(&self, build_id: &str) -> Option<&BuildMetrics> {
        self.builds.get(build_id)
    }

    pub fn get_build_mut(&mut self, build_id: &str) -> Option<&mut BuildMetrics> {
        self.builds.get_mut(build_id)
    }

    pub fn get_recent_builds(&self) -> Vec<&BuildMetrics> {
        self.recent_builds
            .iter()
            .filter_map(|id| self.builds.get(id))
            .collect()
    }

    pub fn get_all_builds(&self) -> Vec<&BuildMetrics> {
        self.builds.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_metrics_creation() {
        let metrics = BuildMetrics::new(
            "test-1".to_string(),
            "test-project".to_string(),
            "rust".to_string(),
        );
        assert_eq!(metrics.build_id, "test-1");
        assert_eq!(metrics.status, BuildStatus::Running);
    }

    #[test]
    fn test_build_metrics_completion() {
        let mut metrics = BuildMetrics::new(
            "test-1".to_string(),
            "test-project".to_string(),
            "rust".to_string(),
        );
        metrics.complete(BuildStatus::Success);
        assert_eq!(metrics.status, BuildStatus::Success);
        assert!(metrics.end_time.is_some());
        assert!(metrics.duration_ms.is_some());
    }

    #[test]
    fn test_metrics_store() {
        let mut store = MetricsStore::new(10);
        let metrics = BuildMetrics::new(
            "test-1".to_string(),
            "test-project".to_string(),
            "rust".to_string(),
        );
        store.add_build(metrics);

        assert!(store.get_build("test-1").is_some());
        assert_eq!(store.get_recent_builds().len(), 1);
    }
}
