#![forbid(unsafe_code)]

pub mod backend;
pub mod backend_utils;
pub mod config;
pub mod diagnostics;
pub mod environment;
pub mod error;
pub mod profiling;
pub mod project;
pub mod security;
pub mod testing;

#[cfg(windows)]
pub mod windows_compat;

pub use backend::BuildBackend;
pub use backend_utils::{
    BinaryUtils, DEFAULT_EXCLUDED_DIRS, FingerprintUtils, TaskDagBuilder, ToolchainUtils,
};
pub use config::{
    BuildConfig, CIConfig, CacheConfig, ConfigError, ExperimentalConfig, ForgeConfig,
    GeneralConfig, SecurityConfig,
};
pub use diagnostics::{
    ComponentHealthMonitor, DiagnosticLog, DiagnosticLogger, DiagnosticReport, HealthCheckRegistry,
    LogLevel, OpPerformanceMetrics, SystemDiagnostics,
};
pub use environment::EnvironmentFingerprint;
pub use error::{
    DiagnosticInfo, ErrorContext, ErrorSeverity, ForgeError, HealthCheck, HealthCheckResult,
    HealthStatus, RecoveryStrategy, Result,
};
pub use profiling::{
    MemoryTracker, PerformanceMetrics, Profiler, ResourceMonitor, ResourceUsage, TaskProfile,
};
pub use security::{
    InputValidator, SecurityError, SecurityLevel, SecurityPolicy, SecurityValidator,
};
pub use testing::{
    Benchmark, BenchmarkResult, IntegrationTestEnvironment, ParallelTestRunner, PropertyTestResult,
    PropertyTestRunner, TestCase, TestResult, TestRunner, TestSuite, TestSummary,
};

#[cfg(windows)]
pub use windows_compat::{
    get_windows_version, is_developer_mode_enabled, is_file_locked, safe_replace_file,
    try_symlink_or_copy,
};
