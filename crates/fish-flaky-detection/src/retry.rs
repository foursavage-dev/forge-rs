// Retry policy and executor

#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_ms: 1000,
        }
    }
}

#[derive(Clone)]
pub struct RetryExecutor {
    policy: RetryPolicy,
}

impl RetryExecutor {
    pub fn new() -> Self {
        Self {
            policy: RetryPolicy::default(),
        }
    }

    pub async fn retry(&self, test_name: &str) -> Result<(), anyhow::Error> {
        // The retry execution is not implemented yet. Failing loudly prevents
        // a caller from believing a flaky test was re-run when it was not.
        Err(anyhow::anyhow!(
            "test retry is not implemented yet (test `{test_name}`; policy allows {} retries)",
            self.policy.max_retries
        ))
    }
}

impl Default for RetryExecutor {
    fn default() -> Self {
        Self::new()
    }
}
