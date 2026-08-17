#![forbid(unsafe_code)]

//! Diagnostic logging and health check system
//!
//! This module provides comprehensive diagnostic logging, health monitoring,
//! and system observability for production readiness.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use crate::error::{HealthCheck, HealthCheckResult, HealthStatus};

/// Diagnostic log entry
#[derive(Debug, Clone)]
pub struct DiagnosticLog {
    pub timestamp: SystemTime,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

impl DiagnosticLog {
    pub fn new(level: LogLevel, component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level,
            component: component.into(),
            message: message.into(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Diagnostic logger
pub struct DiagnosticLogger {
    logs: Arc<RwLock<Vec<DiagnosticLog>>>,
    max_logs: usize,
}

impl DiagnosticLogger {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::with_capacity(max_logs))),
            max_logs,
        }
    }

    pub fn log(&self, level: LogLevel, component: impl Into<String>, message: impl Into<String>) {
        let log = DiagnosticLog::new(level, component, message);
        self.add_log(log);
    }

    pub fn log_with_metadata(
        &self,
        level: LogLevel,
        component: impl Into<String>,
        message: impl Into<String>,
        metadata: HashMap<String, String>,
    ) {
        let log = DiagnosticLog::new(level, component, message);
        let mut log = log;
        log.metadata = metadata;
        self.add_log(log);
    }

    fn add_log(&self, log: DiagnosticLog) {
        if let Ok(mut logs) = self.logs.write() {
            logs.push(log);
            if logs.len() > self.max_logs {
                logs.remove(0);
            }
        }
        // If lock is poisoned, silently fail to avoid crashing
    }

    pub fn get_logs(&self) -> Vec<DiagnosticLog> {
        self.logs.read().map(|logs| logs.clone()).unwrap_or_default()
    }

    pub fn get_logs_by_component(&self, component: &str) -> Vec<DiagnosticLog> {
        self.logs
            .read()
            .map(|logs| {
                logs.iter()
                    .filter(|log| log.component == component)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_logs_by_level(&self, level: LogLevel) -> Vec<DiagnosticLog> {
        self.logs
            .read()
            .map(|logs| {
                logs.iter()
                    .filter(|log| log.level == level)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut logs) = self.logs.write() {
            logs.clear();
        }
    }

    pub fn log_count(&self) -> usize {
        self.logs.read().map(|logs| logs.len()).unwrap_or(0)
    }
}

impl Clone for DiagnosticLogger {
    fn clone(&self) -> Self {
        Self {
            logs: Arc::clone(&self.logs),
            max_logs: self.max_logs,
        }
    }
}

/// Health check registry
pub struct HealthCheckRegistry {
    checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck + Send + Sync>>>>,
}

impl HealthCheckRegistry {
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&self, check: Box<dyn HealthCheck + Send + Sync>) {
        if let Ok(mut checks) = self.checks.write() {
            checks.insert(check.name().to_string(), check);
        }
    }

    pub fn unregister(&self, name: &str) {
        if let Ok(mut checks) = self.checks.write() {
            checks.remove(name);
        }
    }

    pub fn check_all(&self) -> Vec<HealthCheckResult> {
        self.checks
            .read()
            .map(|checks| checks.values().map(|check| check.check_health()).collect())
            .unwrap_or_default()
    }

    pub fn check_component(&self, name: &str) -> Option<HealthCheckResult> {
        self.checks
            .read()
            .ok()
            .and_then(|checks| checks.get(name).map(|check| check.check_health()))
    }

    pub fn get_component_names(&self) -> Vec<String> {
        let checks = self.checks.read().unwrap();
        checks.keys().cloned().collect()
    }

    pub fn overall_health(&self) -> HealthStatus {
        let results = self.check_all();
        if results.is_empty() {
            return HealthStatus::Unknown;
        }

        let has_unhealthy = results.iter().any(|r| r.status == HealthStatus::Unhealthy);
        let has_degraded = results.iter().any(|r| r.status == HealthStatus::Degraded);

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

impl Default for HealthCheckRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Component health monitor
pub struct ComponentHealthMonitor {
    name: String,
    logger: DiagnosticLogger,
    last_check: Arc<RwLock<Option<SystemTime>>>,
    check_count: Arc<RwLock<u64>>,
}

impl ComponentHealthMonitor {
    pub fn new(name: impl Into<String>, logger: DiagnosticLogger) -> Self {
        Self {
            name: name.into(),
            logger,
            last_check: Arc::new(RwLock::new(None)),
            check_count: Arc::new(RwLock::new(0)),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn record_check(&self, result: &HealthCheckResult) {
        *self.last_check.write().unwrap() = Some(SystemTime::now());
        *self.check_count.write().unwrap() += 1;

        let level = match result.status {
            HealthStatus::Healthy => LogLevel::Info,
            HealthStatus::Degraded => LogLevel::Warn,
            HealthStatus::Unhealthy => LogLevel::Error,
            HealthStatus::Unknown => LogLevel::Debug,
        };

        self.logger.log(level, &self.name, &result.message);
    }

    pub fn last_check_time(&self) -> Option<SystemTime> {
        *self.last_check.read().unwrap()
    }

    pub fn check_count(&self) -> u64 {
        *self.check_count.read().unwrap()
    }
}

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct OpPerformanceMetrics {
    pub operation_count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub success_count: u64,
    pub failure_count: u64,
}

impl OpPerformanceMetrics {
    pub fn new() -> Self {
        Self {
            operation_count: 0,
            total_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            success_count: 0,
            failure_count: 0,
        }
    }

    pub fn record_operation(&mut self, duration: Duration, success: bool) {
        self.operation_count += 1;
        self.total_duration += duration;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);

        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }
    }

    pub fn average_duration(&self) -> Duration {
        if self.operation_count == 0 {
            Duration::ZERO
        } else {
            self.total_duration / self.operation_count as u32
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.operation_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.operation_count as f64
        }
    }
}

impl Default for OpPerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// System diagnostics aggregator
pub struct SystemDiagnostics {
    logger: DiagnosticLogger,
    health_registry: HealthCheckRegistry,
    performance_metrics: Arc<RwLock<HashMap<String, OpPerformanceMetrics>>>,
}

impl SystemDiagnostics {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logger: DiagnosticLogger::new(max_logs),
            health_registry: HealthCheckRegistry::new(),
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn logger(&self) -> DiagnosticLogger {
        self.logger.clone()
    }

    pub fn health_registry(&self) -> HealthCheckRegistry {
        HealthCheckRegistry {
            checks: Arc::clone(&self.health_registry.checks),
        }
    }

    pub fn record_performance(
        &self,
        operation: impl Into<String>,
        duration: Duration,
        success: bool,
    ) {
        let mut metrics = self.performance_metrics.write().unwrap();
        let operation = operation.into();
        let entry = metrics.entry(operation).or_default();
        entry.record_operation(duration, success);
    }

    pub fn get_performance_metrics(&self, operation: &str) -> Option<OpPerformanceMetrics> {
        let metrics = self.performance_metrics.read().unwrap();
        metrics.get(operation).cloned()
    }

    pub fn all_performance_metrics(&self) -> HashMap<String, OpPerformanceMetrics> {
        self.performance_metrics.read().unwrap().clone()
    }

    pub fn generate_report(&self) -> DiagnosticReport {
        let health_results = self.health_registry.check_all();
        let overall_health = self.health_registry.overall_health();
        let performance_metrics = self.all_performance_metrics();
        let logs = self.logger.get_logs();

        DiagnosticReport {
            timestamp: SystemTime::now(),
            overall_health,
            health_results,
            performance_metrics,
            log_count: logs.len(),
            error_count: logs.iter().filter(|l| l.level == LogLevel::Error).count(),
            warning_count: logs.iter().filter(|l| l.level == LogLevel::Warn).count(),
        }
    }
}

/// Diagnostic report
#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    pub timestamp: SystemTime,
    pub overall_health: HealthStatus,
    pub health_results: Vec<HealthCheckResult>,
    pub performance_metrics: HashMap<String, OpPerformanceMetrics>,
    pub log_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
}

impl DiagnosticReport {
    pub fn to_summary(&self) -> String {
        format!(
            "System Diagnostics Report\n\
             ========================\n\
             Timestamp: {:?}\n\
             Overall Health: {}\n\
             Components Checked: {}\n\
             Log Entries: {}\n\
             Errors: {}\n\
             Warnings: {}\n\
             Performance Metrics: {}",
            self.timestamp,
            self.overall_health,
            self.health_results.len(),
            self.log_count,
            self.error_count,
            self.warning_count,
            self.performance_metrics.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_logger() {
        let logger = DiagnosticLogger::new(100);
        logger.log(LogLevel::Info, "test", "Test message");

        assert_eq!(logger.log_count(), 1);

        let logs = logger.get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, LogLevel::Info);
        assert_eq!(logs[0].component, "test");
        assert_eq!(logs[0].message, "Test message");
    }

    #[test]
    fn test_diagnostic_logger_max_logs() {
        let logger = DiagnosticLogger::new(5);
        for i in 0..10 {
            logger.log(LogLevel::Info, "test", format!("Message {}", i));
        }

        assert_eq!(logger.log_count(), 5);
    }

    #[test]
    fn test_health_check_registry() {
        let registry = HealthCheckRegistry::new();
        assert_eq!(registry.overall_health(), HealthStatus::Unknown);
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = OpPerformanceMetrics::new();
        metrics.record_operation(Duration::from_millis(100), true);
        metrics.record_operation(Duration::from_millis(200), true);
        metrics.record_operation(Duration::from_millis(50), false);

        assert_eq!(metrics.operation_count, 3);
        assert_eq!(metrics.success_count, 2);
        assert_eq!(metrics.failure_count, 1);
        assert_eq!(metrics.min_duration, Duration::from_millis(50));
        assert_eq!(metrics.max_duration, Duration::from_millis(200));
    }

    #[test]
    fn test_performance_metrics_average() {
        let mut metrics = OpPerformanceMetrics::new();
        metrics.record_operation(Duration::from_millis(100), true);
        metrics.record_operation(Duration::from_millis(200), true);

        let avg = metrics.average_duration();
        assert_eq!(avg, Duration::from_millis(150));
    }

    #[test]
    fn test_performance_metrics_success_rate() {
        let mut metrics = OpPerformanceMetrics::new();
        metrics.record_operation(Duration::from_millis(100), true);
        metrics.record_operation(Duration::from_millis(200), false);
        metrics.record_operation(Duration::from_millis(150), true);

        let rate = metrics.success_rate();
        assert!((rate - 0.6666).abs() < 0.01);
    }

    #[test]
    fn test_system_diagnostics() {
        let diagnostics = SystemDiagnostics::new(100);
        let logger = diagnostics.logger();

        logger.log(LogLevel::Info, "test", "Test message");
        logger.log(LogLevel::Error, "test", "Error message");
        logger.log(LogLevel::Warn, "test", "Warning message");

        diagnostics.record_performance("test_op", Duration::from_millis(100), true);

        let report = diagnostics.generate_report();
        assert_eq!(report.log_count, 3);
        assert_eq!(report.error_count, 1);
        assert_eq!(report.warning_count, 1);
        assert_eq!(report.performance_metrics.len(), 1);
    }
}
