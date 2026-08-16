#![forbid(unsafe_code)]

//! Enhanced error handling with structured context
//!
//! This module provides comprehensive error handling with structured context,
//! recovery strategies, and diagnostic information for production readiness.

use thiserror::Error;

/// Structured error context for better diagnostics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorContext {
    /// Operation that failed
    pub operation: String,
    /// Component/subsystem
    pub component: String,
    /// File where error occurred
    pub file: Option<String>,
    /// Line number where error occurred
    pub line: Option<u32>,
    /// Suggested fixes or recovery actions
    pub suggestions: Vec<String>,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Additional context
    pub metadata: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            component: component.into(),
            file: None,
            line: None,
            suggestions: Vec::new(),
            severity: ErrorSeverity::Error,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ErrorSeverity {
    /// Informational only
    Info,
    /// Warning that doesn't prevent operation
    Warning,
    /// Error that prevents operation
    Error,
    /// Critical error that needs immediate attention
    Critical,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARN"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Enhanced error with structured context
#[derive(Error, Debug)]
pub struct ForgeError {
    kind: ErrorKind,
    context: Box<ErrorContext>,
}

impl ForgeError {
    pub fn new(kind: ErrorKind, context: ErrorContext) -> Self {
        Self { kind, context: Box::new(context) }
    }

    pub fn context(&self) -> &ErrorContext {
        &self.context
    }

    pub fn operation(&self) -> &str {
        &self.context.operation
    }

    pub fn component(&self) -> &str {
        &self.context.component
    }

    pub fn suggestions(&self) -> &[String] {
        &self.context.suggestions
    }

    pub fn severity(&self) -> ErrorSeverity {
        self.context.severity
    }

    pub fn is_critical(&self) -> bool {
        self.context.severity == ErrorSeverity::Critical
    }

    pub fn is_recoverable(&self) -> bool {
        matches!(self.kind, ErrorKind::Recoverable(_))
    }
}

impl std::fmt::Display for ForgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} in {}: {}",
            self.context.severity, self.context.operation, self.context.component, self.kind
        )?;

        if !self.context.suggestions.is_empty() {
            write!(f, "\nSuggestions: {}", self.context.suggestions.join(", "))?;
        }

        Ok(())
    }
}

/// Error kinds with recovery information
#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Graph error: {0}")]
    Graph(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Executor error: {0}")]
    Executor(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Recoverable: {0}")]
    Recoverable(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Non-UTF-8 manifest path: {0}")]
    NonUtf8ManifestPath(String),

    #[error("Cargo metadata error: {0}")]
    CargoMetadata(String),
}

/// Error recovery strategy
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation with exponential backoff
    Retry { max_attempts: u32, backoff_ms: u64 },
    /// Use fallback implementation
    Fallback { alternative: String },
    /// Skip the operation and continue
    Skip { reason: String },
    /// Abort the operation
    Abort,
    /// Graceful degradation
    Degrade { alternative: String },
}

impl RecoveryStrategy {
    pub fn from_error(error: &ForgeError) -> Self {
        match error.kind {
            ErrorKind::Recoverable(_) => RecoveryStrategy::Retry {
                max_attempts: 3,
                backoff_ms: 1000,
            },
            ErrorKind::Network(_) => RecoveryStrategy::Retry {
                max_attempts: 5,
                backoff_ms: 2000,
            },
            ErrorKind::Timeout(_) => RecoveryStrategy::Retry {
                max_attempts: 2,
                backoff_ms: 500,
            },
            ErrorKind::Io(_) => RecoveryStrategy::Retry {
                max_attempts: 3,
                backoff_ms: 1000,
            },
            _ => RecoveryStrategy::Abort,
        }
    }
}

/// Diagnostic information for errors
#[derive(Debug, Clone, serde::Serialize)]
pub struct DiagnosticInfo {
    pub error_id: String,
    pub timestamp: String,
    pub error_type: String,
    pub component: String,
    pub operation: String,
    pub severity: ErrorSeverity,
    pub suggestions: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl DiagnosticInfo {
    pub fn from_error(error: &ForgeError) -> Self {
        Self {
            error_id: uuid::Uuid::new_v4().to_string(),
            timestamp: format!("{:?}", std::time::SystemTime::now()),
            error_type: format!("{:?}", error.kind),
            component: error.context.component.clone(),
            operation: error.context.operation.clone(),
            severity: error.context.severity,
            suggestions: error.context.suggestions.clone(),
            metadata: error.context.metadata.clone(),
        }
    }

    pub fn to_json(&self) -> std::result::Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Health check result
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Health check result with details
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub component: String,
    pub message: String,
    pub timestamp: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl HealthCheckResult {
    pub fn healthy(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            component: component.into(),
            message: message.into(),
            timestamp: format!("{:?}", std::time::SystemTime::now()),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn degraded(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            component: component.into(),
            message: message.into(),
            timestamp: format!("{:?}", std::time::SystemTime::now()),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn unhealthy(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            component: component.into(),
            message: message.into(),
            timestamp: format!("{:?}", std::time::SystemTime::now()),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Trait for components that can be health-checked
pub trait HealthCheck {
    fn check_health(&self) -> HealthCheckResult;
    fn name(&self) -> &str;
}

/// Result type alias for Forge operations
pub type Result<T> = std::result::Result<T, ForgeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new("test_operation", "test_component")
            .with_file("test.rs")
            .with_line(42)
            .with_suggestion("Check your configuration")
            .with_severity(ErrorSeverity::Warning);

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.component, "test_component");
        assert_eq!(context.file, Some("test.rs".to_string()));
        assert_eq!(context.line, Some(42));
        assert_eq!(context.suggestions.len(), 1);
        assert_eq!(context.severity, ErrorSeverity::Warning);
    }

    #[test]
    fn test_forge_error_creation() {
        let error = ForgeError::new(
            ErrorKind::Config("Invalid configuration".to_string()),
            ErrorContext::new("load_config", "config_loader")
                .with_file("config.toml")
                .with_line(10)
                .with_suggestion("Check configuration syntax")
                .with_severity(ErrorSeverity::Error),
        );

        assert_eq!(error.operation(), "load_config");
        assert_eq!(error.component(), "config_loader");
        assert_eq!(error.severity(), ErrorSeverity::Error);
        assert!(!error.is_critical());
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_diagnostic_info_generation() {
        let error = ForgeError::new(
            ErrorKind::Recoverable("Temporary failure".to_string()),
            ErrorContext::new("cache_read", "cache").with_severity(ErrorSeverity::Warning),
        );

        let diagnostic = DiagnosticInfo::from_error(&error);
        assert_eq!(diagnostic.component, "cache");
        assert_eq!(diagnostic.operation, "cache_read");
        assert_eq!(diagnostic.severity, ErrorSeverity::Warning);
        assert!(!diagnostic.error_id.is_empty());
    }

    #[test]
    fn test_health_check_results() {
        let healthy = HealthCheckResult::healthy("test", "All good");
        assert_eq!(healthy.status, HealthStatus::Healthy);

        let degraded = HealthCheckResult::degraded("test", "Running slowly");
        assert_eq!(degraded.status, HealthStatus::Degraded);

        let unhealthy = HealthCheckResult::unhealthy("test", "Failed");
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_recovery_strategy_selection() {
        let recoverable_error = ForgeError::new(
            ErrorKind::Recoverable("Temporary failure".to_string()),
            ErrorContext::new("cache_get", "cache"),
        );

        let strategy = RecoveryStrategy::from_error(&recoverable_error);
        match strategy {
            RecoveryStrategy::Retry {
                max_attempts,
                backoff_ms,
            } => {
                assert_eq!(max_attempts, 3);
                assert_eq!(backoff_ms, 1000);
            }
            _ => panic!("Expected retry strategy"),
        }
    }
}
