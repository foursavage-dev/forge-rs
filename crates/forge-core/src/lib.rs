#![forbid(unsafe_code)]

pub mod backend;
pub mod backend_utils;
pub mod config;
pub mod environment;
pub mod error;
pub mod profiling;
pub mod project;
pub mod security;
pub mod testing;

#[cfg(windows)]
pub mod windows_compat;

pub use backend::BuildBackend;
pub use backend_utils::{BinaryUtils, FingerprintUtils, TaskDagBuilder, ToolchainUtils, DEFAULT_EXCLUDED_DIRS};
pub use config::{ForgeConfig, GeneralConfig, CacheConfig, BuildConfig, CIConfig, SecurityConfig, ExperimentalConfig, ConfigError};
pub use environment::EnvironmentFingerprint;
pub use error::{ForgeError, Result};
pub use profiling::{Profiler, PerformanceMetrics, TaskProfile, MemoryTracker, ResourceMonitor, ResourceUsage};
pub use security::{SecurityPolicy, SecurityValidator, SecurityError, SecurityLevel, InputValidator};
pub use testing::{TestRunner, TestSuite, TestResult, TestSummary, PropertyTestRunner, PropertyTestResult, IntegrationTestEnvironment, Benchmark, BenchmarkResult, ParallelTestRunner, TestCase};

#[cfg(windows)]
pub use windows_compat::{try_symlink_or_copy, is_file_locked, safe_replace_file, get_windows_version, is_developer_mode_enabled};
