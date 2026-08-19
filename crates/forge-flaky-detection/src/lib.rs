// Forge Flaky Detection - Flaky Test Detection & Auto-Retry
// Detects flaky tests with statistical analysis and auto-retry

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod detector;
pub mod history;
pub mod retry;

pub use detector::{FlakyDetector, FlakyTest};
pub use history::TestHistory;
pub use retry::{RetryExecutor, RetryPolicy};

/// Main flaky detection service
pub struct FlakyDetectionService {
    detector: FlakyDetector,
    retry_executor: RetryExecutor,
}

impl FlakyDetectionService {
    pub fn new() -> Self {
        Self {
            detector: FlakyDetector::new(),
            retry_executor: RetryExecutor::new(),
        }
    }

    pub async fn detect_and_retry(
        &self,
        test_name: &str,
    ) -> Result<bool, anyhow::Error> {
        let is_flaky = self.detector.is_flaky(test_name).await?;
        if is_flaky {
            self.retry_executor.retry(test_name).await?;
        }
        Ok(is_flaky)
    }
}

impl Default for FlakyDetectionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_flaky_detection_service() {
        let service = FlakyDetectionService::new();
        let result = service.detect_and_retry("tests::test_cache").await.unwrap();
        assert!(!result);
    }

    #[test]
    fn test_retry_policy_defaults() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.backoff_ms, 1000);
    }

    #[test]
    fn test_flaky_test_serialization() {
        let flaky = FlakyTest {
            test_name: "tests::network_sync".to_string(),
            failure_rate: 0.25,
            total_runs: 20,
            failed_runs: 5,
            last_detected: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&flaky).unwrap();
        assert!(json.contains("tests::network_sync"));
        assert!(json.contains("0.25"));
    }
}
