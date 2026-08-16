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
    ) -> Result<bool, Box<dyn std::error::Error>> {
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
