use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    TimedOut,
}

#[derive(Debug, Clone)]
pub struct SingleTestResult {
    pub name: String,
    pub status: TestStatus,
    pub duration: Duration,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct NextestRunner {
    pub max_parallel_jobs: usize,
    pub per_test_timeout: Duration,
    pub retry_count: usize,
}

impl NextestRunner {
    pub fn new(max_parallel_jobs: usize) -> Self {
        Self {
            max_parallel_jobs,
            per_test_timeout: Duration::from_secs(60),
            retry_count: 1,
        }
    }

    pub fn parse_terse_test_list(raw_output: &str) -> Vec<String> {
        let mut tests = Vec::new();
        for line in raw_output.lines() {
            let line = line.trim();
            if line.ends_with(": test") {
                let test_name = line.trim_end_matches(": test").trim();
                tests.push(test_name.to_string());
            } else if !line.ends_with(": benchmark") && !line.is_empty() && !line.ends_with("tests")
            {
                tests.push(line.to_string());
            }
        }
        tests
    }

    pub fn aggregate_summary(results: &[SingleTestResult]) -> (usize, usize, usize, usize) {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        let mut timed_out = 0;

        for r in results {
            match r.status {
                TestStatus::Passed => passed += 1,
                TestStatus::Failed => failed += 1,
                TestStatus::Skipped => skipped += 1,
                TestStatus::TimedOut => timed_out += 1,
            }
        }

        (passed, failed, skipped, timed_out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_terse_test_list() {
        let output = "tests::test_foo: test\ntests::test_bar: test\ntests::bench_baz: benchmark\n";
        let list = NextestRunner::parse_terse_test_list(output);
        assert_eq!(list, vec!["tests::test_foo", "tests::test_bar"]);
    }

    #[test]
    fn test_aggregate_summary() {
        let results = vec![
            SingleTestResult {
                name: "t1".to_string(),
                status: TestStatus::Passed,
                duration: Duration::from_millis(10),
                stdout: String::new(),
                stderr: String::new(),
            },
            SingleTestResult {
                name: "t2".to_string(),
                status: TestStatus::Failed,
                duration: Duration::from_millis(20),
                stdout: String::new(),
                stderr: "assertion failed".to_string(),
            },
        ];

        let (passed, failed, skipped, timed_out) = NextestRunner::aggregate_summary(&results);
        assert_eq!(passed, 1);
        assert_eq!(failed, 1);
        assert_eq!(skipped, 0);
        assert_eq!(timed_out, 0);
    }
}
