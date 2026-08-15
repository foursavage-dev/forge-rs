#![forbid(unsafe_code)]

//! Comprehensive testing framework
//! 
//! This module provides testing utilities and frameworks for validating
//! build system functionality, including unit tests, integration tests,
//! and property-based testing.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};
use spin::Mutex as SpinMutex;

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration: std::time::Duration,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TestSuite {
    pub name: String,
    pub tests: Vec<TestResult>,
    pub total_duration: std::time::Duration,
}

pub struct TestRunner {
    suites: Arc<SpinMutex<HashMap<String, TestSuite>>>,
    total_tests: Arc<AtomicU64>,
    passed_tests: Arc<AtomicU64>,
    failed_tests: Arc<AtomicU64>,
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            suites: Arc::new(SpinMutex::new(HashMap::new())),
            total_tests: Arc::new(AtomicU64::new(0)),
            passed_tests: Arc::new(AtomicU64::new(0)),
            failed_tests: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn run_test<F>(&self, _suite_name: String, test_name: String, test_fn: F) -> TestResult
    where
        F: FnOnce() -> Result<(), Box<dyn std::error::Error>>,
    {
        let start = std::time::Instant::now();
        let result = test_fn();
        let duration = start.elapsed();
        
        let passed = result.is_ok();
        let error_message = result.err().map(|e| e.to_string());
        
        self.total_tests.fetch_add(1, Ordering::SeqCst);
        if passed {
            self.passed_tests.fetch_add(1, Ordering::SeqCst);
        } else {
            self.failed_tests.fetch_add(1, Ordering::SeqCst);
        }
        
        TestResult {
            name: test_name,
            passed,
            duration,
            error_message,
        }
    }

    pub fn add_suite(&self, suite: TestSuite) {
        let mut suites = self.suites.lock();
        suites.insert(suite.name.clone(), suite);
    }

    pub fn get_summary(&self) -> TestSummary {
        let total = self.total_tests.load(Ordering::SeqCst);
        let passed = self.passed_tests.load(Ordering::SeqCst);
        let failed = self.failed_tests.load(Ordering::SeqCst);
        
        let suites = self.suites.lock();
        let suite_count = suites.len();
        
        TestSummary {
            total_tests: total,
            passed_tests: passed,
            failed_tests: failed,
            success_rate: if total > 0 { passed as f64 / total as f64 } else { 0.0 },
            suite_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestSummary {
    pub total_tests: u64,
    pub passed_tests: u64,
    pub failed_tests: u64,
    pub success_rate: f64,
    pub suite_count: usize,
}

/// Property-based testing utilities
pub struct PropertyTestRunner {
    iterations: u64,
    max_iterations: u64,
}

impl PropertyTestRunner {
    pub fn new(max_iterations: u64) -> Self {
        Self {
            iterations: 0,
            max_iterations,
        }
    }

    pub fn run_property<F>(&mut self, property_name: &str, mut test_fn: F) -> PropertyTestResult
    where
        F: FnMut() -> bool,
    {
        let mut passed = 0;
        let mut failed = 0;
        let mut counter_example = None;
        
        for i in 0..self.max_iterations {
            self.iterations = i + 1;
            
            if test_fn() {
                passed += 1;
            } else {
                failed += 1;
                counter_example = Some(format!("Iteration {}", i));
                break;
            }
        }
        
        PropertyTestResult {
            property_name: property_name.to_string(),
            iterations: self.iterations,
            passed,
            failed,
            counter_example,
            success: failed == 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PropertyTestResult {
    pub property_name: String,
    pub iterations: u64,
    pub passed: u64,
    pub failed: u64,
    pub counter_example: Option<String>,
    pub success: bool,
}

/// Integration test utilities
pub struct IntegrationTestEnvironment {
    temp_dir: Option<tempfile::TempDir>,
    pub environment_vars: HashMap<String, String>,
}

impl Default for IntegrationTestEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl IntegrationTestEnvironment {
    pub fn new() -> Self {
        Self {
            temp_dir: tempfile::TempDir::new().ok(),
            environment_vars: HashMap::new(),
        }
    }

    pub fn temp_dir(&self) -> Option<&Path> {
        self.temp_dir.as_ref().map(|d| d.path())
    }

    pub fn set_env_var(&mut self, key: String, value: String) {
        self.environment_vars.insert(key, value);
    }

    pub fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        // In a safe implementation without unsafe code, we cannot set environment variables
        // Instead, we validate that the environment variables are properly stored
        // and can be used by the test framework in other ways
        // The environment_vars map serves as a testable record of what would be set
        Ok(())
    }

    pub fn teardown(&self) {
        // In a safe implementation without unsafe code, we cannot remove environment variables
        // The environment_vars map is cleared when the IntegrationTestEnvironment is dropped
    }
}

impl Drop for IntegrationTestEnvironment {
    fn drop(&mut self) {
        self.teardown();
    }
}

/// Benchmark utilities
pub struct Benchmark {
    name: String,
    iterations: u64,
}

impl Benchmark {
    pub fn new(name: String, iterations: u64) -> Self {
        Self { name, iterations }
    }

    pub fn run<F>(&self, benchmark_fn: F) -> BenchmarkResult
    where
        F: Fn(),
    {
        let start = std::time::Instant::now();
        
        for _ in 0..self.iterations {
            benchmark_fn();
        }
        
        let duration = start.elapsed();
        let avg_duration = duration / self.iterations as u32;
        
        BenchmarkResult {
            name: self.name.clone(),
            iterations: self.iterations,
            total_duration: duration,
            average_duration: avg_duration,
            ops_per_second: self.iterations as f64 / duration.as_secs_f64(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: u64,
    pub total_duration: std::time::Duration,
    pub average_duration: std::time::Duration,
    pub ops_per_second: f64,
}

impl Clone for TestRunner {
    fn clone(&self) -> Self {
        Self {
            suites: self.suites.clone(),
            total_tests: self.total_tests.clone(),
            passed_tests: self.passed_tests.clone(),
            failed_tests: self.failed_tests.clone(),
        }
    }
}

/// Simple semaphore for limiting concurrent operations
struct Semaphore {
    permits: Arc<(Mutex<usize>, Condvar)>,
}

impl Semaphore {
    fn new(permits: usize) -> Self {
        Self {
            permits: Arc::new((Mutex::new(permits), Condvar::new())),
        }
    }

    fn acquire(&self) {
        let (lock, cvar) = &*self.permits;
        let mut permits = lock.lock().unwrap();
        while *permits == 0 {
            permits = cvar.wait(permits).unwrap();
        }
        *permits -= 1;
    }

    fn release(&self) {
        let (lock, cvar) = &*self.permits;
        let mut permits = lock.lock().unwrap();
        *permits += 1;
        cvar.notify_one();
    }
}

/// Type alias for test case to reduce type complexity
pub type TestCase = (String, String, Box<dyn FnOnce() -> Result<(), Box<dyn std::error::Error>> + Send>);

/// Parallel test runner for concurrent test execution
pub struct ParallelTestRunner {
    total_tests: Arc<AtomicU64>,
    passed_tests: Arc<AtomicU64>,
    failed_tests: Arc<AtomicU64>,
    max_parallel: usize,
}

impl ParallelTestRunner {
    pub fn new(max_parallel: usize) -> Self {
        Self {
            total_tests: Arc::new(AtomicU64::new(0)),
            passed_tests: Arc::new(AtomicU64::new(0)),
            failed_tests: Arc::new(AtomicU64::new(0)),
            max_parallel,
        }
    }

    pub fn run_parallel(&self, test_cases: Vec<TestCase>) -> Vec<TestResult>
    {
        use std::sync::mpsc;

        let (tx, rx) = mpsc::channel();
        let semaphore = Arc::new(Semaphore::new(self.max_parallel));
        let total_tests = self.total_tests.clone();
        let passed_tests = self.passed_tests.clone();
        let failed_tests = self.failed_tests.clone();

        for (_suite_name, test_name, test_fn) in test_cases {
            let tx_clone = tx.clone();
            let semaphore_clone = semaphore.clone();
            let total_tests_clone = total_tests.clone();
            let passed_tests_clone = passed_tests.clone();
            let failed_tests_clone = failed_tests.clone();

            std::thread::spawn(move || {
                // Acquire semaphore permit (blocks until available)
                semaphore_clone.acquire();

                let start = std::time::Instant::now();
                let result = test_fn();
                let duration = start.elapsed();

                let passed = result.is_ok();
                let error_message = result.err().map(|e| e.to_string());

                total_tests_clone.fetch_add(1, Ordering::SeqCst);
                if passed {
                    passed_tests_clone.fetch_add(1, Ordering::SeqCst);
                } else {
                    failed_tests_clone.fetch_add(1, Ordering::SeqCst);
                }

                let test_result = TestResult {
                    name: test_name,
                    passed,
                    duration,
                    error_message,
                };

                tx_clone.send(test_result).unwrap();

                // Release semaphore permit
                semaphore_clone.release();
            });
        }

        drop(tx);

        rx.iter().collect()
    }

    pub fn get_summary(&self) -> TestSummary {
        let total = self.total_tests.load(Ordering::SeqCst);
        let passed = self.passed_tests.load(Ordering::SeqCst);
        let failed = self.failed_tests.load(Ordering::SeqCst);

        TestSummary {
            total_tests: total,
            passed_tests: passed,
            failed_tests: failed,
            success_rate: if total > 0 { passed as f64 / total as f64 } else { 0.0 },
            suite_count: 0, // Parallel runner doesn't track suites
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_runner() {
        let runner = TestRunner::new();
        
        let result = runner.run_test("test_suite".to_string(), "test_case".to_string(), || {
            Ok(())
        });
        
        assert!(result.passed);
        assert!(result.error_message.is_none());
        
        let summary = runner.get_summary();
        assert_eq!(summary.total_tests, 1);
        assert_eq!(summary.passed_tests, 1);
    }

    #[test]
    fn test_property_test() {
        let mut runner = PropertyTestRunner::new(100);
        
        let result = runner.run_property("always_true", || true);
        
        assert!(result.success);
        assert_eq!(result.iterations, 100);
        assert_eq!(result.passed, 100);
    }

    #[test]
    fn test_integration_environment() {
        let mut env = IntegrationTestEnvironment::new();
        assert!(env.temp_dir().is_some());

        // Test setting environment variables
        env.set_env_var("FORGE_TEST_VAR".to_string(), "test_value".to_string());
        
        // Verify the structure stores the variables correctly
        assert!(env.environment_vars.contains_key("FORGE_TEST_VAR"));
        assert_eq!(env.environment_vars.get("FORGE_TEST_VAR"), Some(&"test_value".to_string()));
        
        // Setup and teardown should not panic
        env.setup().unwrap();
        env.teardown();
    }

    #[test]
    fn test_benchmark() {
        let benchmark = Benchmark::new("simple_benchmark".to_string(), 1000);

        let result = benchmark.run(|| {
            let _ = 1 + 1;
        });

        assert_eq!(result.iterations, 1000);
        assert!(result.ops_per_second > 0.0);
    }

    #[test]
    fn test_parallel_test_runner() {
        let runner = ParallelTestRunner::new(2);

        let test_cases: Vec<TestCase> = vec![
            ("suite1".to_string(), "test1".to_string(), Box::new(|| Ok::<(), Box<dyn std::error::Error>>(()))),
            ("suite1".to_string(), "test2".to_string(), Box::new(|| Ok::<(), Box<dyn std::error::Error>>(()))),
            ("suite1".to_string(), "test3".to_string(), Box::new(|| Ok::<(), Box<dyn std::error::Error>>(()))),
        ];

        let results = runner.run_parallel(test_cases);

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.passed));

        let summary = runner.get_summary();
        assert_eq!(summary.total_tests, 3);
        assert_eq!(summary.passed_tests, 3);
    }
}

