use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakyTest {
    pub test_name: String,
    pub failure_rate: f64,
    pub total_runs: u32,
    pub failed_runs: u32,
    pub flip_count: u32,
    pub last_detected: DateTime<Utc>,
}

#[derive(Clone, Default)]
pub struct FlakyDetector {
    history: Arc<RwLock<HashMap<String, Vec<bool>>>>,
}

impl FlakyDetector {
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn record_run(&self, test_name: &str, passed: bool) {
        let mut map = self.history.write().unwrap();
        map.entry(test_name.to_string()).or_default().push(passed);
    }

    pub async fn is_flaky(&self, test_name: &str) -> Result<bool, anyhow::Error> {
        let map = self.history.read().unwrap();
        if let Some(runs) = map.get(test_name) {
            if runs.len() < 2 {
                return Ok(false);
            }

            let mut flips = 0;
            let mut failed = 0;
            for i in 0..runs.len() {
                if !runs[i] {
                    failed += 1;
                }
                if i > 0 && runs[i] != runs[i - 1] {
                    flips += 1;
                }
            }

            let flip_rate = flips as f64 / (runs.len() - 1) as f64;
            let failure_rate = failed as f64 / runs.len() as f64;

            Ok(flip_rate > 0.25 || (failure_rate > 0.0 && failure_rate < 1.0))
        } else {
            Ok(false)
        }
    }

    pub fn get_flaky_report(&self, test_name: &str) -> Option<FlakyTest> {
        let map = self.history.read().unwrap();
        let runs = map.get(test_name)?;
        if runs.is_empty() {
            return None;
        }

        let mut flips = 0;
        let mut failed = 0;
        for i in 0..runs.len() {
            if !runs[i] {
                failed += 1;
            }
            if i > 0 && runs[i] != runs[i - 1] {
                flips += 1;
            }
        }

        Some(FlakyTest {
            test_name: test_name.to_string(),
            failure_rate: failed as f64 / runs.len() as f64,
            total_runs: runs.len() as u32,
            failed_runs: failed,
            flip_count: flips,
            last_detected: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_flaky_detection_logic() {
        let detector = FlakyDetector::new();
        detector.record_run("test_network", true);
        detector.record_run("test_network", false);
        detector.record_run("test_network", true);

        assert!(detector.is_flaky("test_network").await.unwrap());

        detector.record_run("test_stable", true);
        detector.record_run("test_stable", true);
        detector.record_run("test_stable", true);

        assert!(!detector.is_flaky("test_stable").await.unwrap());
    }
}
