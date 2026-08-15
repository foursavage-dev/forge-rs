// Test history tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestHistory {
    pub test_name: String,
    pub runs: Vec<TestRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRun {
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub duration_ms: u64,
}
