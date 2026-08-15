#![forbid(unsafe_code)]

//! Security features and validation
//! 
//! This module provides security-related functionality including input validation,
//! permission checks, and security policies.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityLevel {
    AllowAll,
    Strict,
    Paranoid,
}

#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub level: SecurityLevel,
    pub allowed_executables: HashSet<String>,
    pub allowed_paths: HashSet<PathBuf>,
    pub max_file_size: Option<u64>,
    pub max_execution_time: Option<u64>,
}

impl SecurityPolicy {
    pub fn new(level: SecurityLevel) -> Self {
        Self {
            level,
            allowed_executables: HashSet::new(),
            allowed_paths: HashSet::new(),
            max_file_size: None,
            max_execution_time: None,
        }
    }

    pub fn add_allowed_executable(&mut self, executable: String) {
        self.allowed_executables.insert(executable);
    }

    pub fn add_allowed_path(&mut self, path: PathBuf) {
        self.allowed_paths.insert(path);
    }

    pub fn set_max_file_size(&mut self, size: u64) {
        self.max_file_size = Some(size);
    }

    pub fn set_max_execution_time(&mut self, seconds: u64) {
        self.max_execution_time = Some(seconds);
    }

    pub fn is_executable_allowed(&self, executable: &str) -> bool {
        match self.level {
            SecurityLevel::AllowAll => true,
            SecurityLevel::Strict => self.allowed_executables.contains(executable),
            SecurityLevel::Paranoid => self.allowed_executables.contains(executable),
        }
    }

    pub fn is_path_allowed(&self, path: &Path) -> bool {
        match self.level {
            SecurityLevel::AllowAll => true,
            SecurityLevel::Strict => {
                self.allowed_paths.is_empty() || self.allowed_paths.iter().any(|allowed| path.starts_with(allowed))
            }
            SecurityLevel::Paranoid => {
                self.allowed_paths.iter().any(|allowed| path.starts_with(allowed))
            }
        }
    }

    pub fn is_file_size_allowed(&self, size: u64) -> bool {
        match self.max_file_size {
            Some(max) => size <= max,
            None => true,
        }
    }

    pub fn is_execution_time_allowed(&self, seconds: u64) -> bool {
        match self.max_execution_time {
            Some(max) => seconds <= max,
            None => true,
        }
    }
}

pub struct SecurityValidator {
    policy: Arc<RwLock<SecurityPolicy>>,
}

impl SecurityValidator {
    pub fn new(policy: SecurityPolicy) -> Self {
        Self {
            policy: Arc::new(RwLock::new(policy)),
        }
    }

    pub fn validate_command(&self, command: &str, args: &[String]) -> Result<(), SecurityError> {
        let policy = self.policy.read().unwrap();
        
        // Extract executable from command
        let executable = command.split_whitespace().next()
            .ok_or_else(|| SecurityError::InvalidCommand("Empty command".to_string()))?;
        
        if !policy.is_executable_allowed(executable) {
            return Err(SecurityError::ExecutableNotAllowed(executable.to_string()));
        }

        // Validate arguments for suspicious patterns
        for arg in args {
            if self.is_suspicious_argument(arg) {
                return Err(SecurityError::SuspiciousArgument(arg.clone()));
            }
        }

        Ok(())
    }

    pub fn validate_path(&self, path: &Path) -> Result<(), SecurityError> {
        let policy = self.policy.read().unwrap();
        
        if !policy.is_path_allowed(path) {
            return Err(SecurityError::PathNotAllowed(path.to_path_buf()));
        }

        // Check for path traversal attempts
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            return Err(SecurityError::PathTraversalAttempt(path.to_path_buf()));
        }

        Ok(())
    }

    pub fn validate_file_size(&self, size: u64) -> Result<(), SecurityError> {
        let policy = self.policy.read().unwrap();
        
        if !policy.is_file_size_allowed(size) {
            return Err(SecurityError::FileSizeExceeded(size));
        }

        Ok(())
    }

    fn is_suspicious_argument(&self, arg: &str) -> bool {
        let suspicious_patterns = [
            ";", "&", "|", "$(", "`",
            "&&", "||", "<", ">", ">>", "<<",
        ];

        suspicious_patterns.iter().any(|pattern| arg.contains(pattern))
    }

    pub fn update_policy(&self, new_policy: SecurityPolicy) {
        let mut policy = self.policy.write().unwrap();
        *policy = new_policy;
    }
}

#[derive(Debug, Clone)]
pub enum SecurityError {
    InvalidCommand(String),
    ExecutableNotAllowed(String),
    PathNotAllowed(PathBuf),
    PathTraversalAttempt(PathBuf),
    FileSizeExceeded(u64),
    SuspiciousArgument(String),
    ExecutionTimeout,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::InvalidCommand(msg) => write!(f, "Invalid command: {}", msg),
            SecurityError::ExecutableNotAllowed(exec) => write!(f, "Executable not allowed: {}", exec),
            SecurityError::PathNotAllowed(path) => write!(f, "Path not allowed: {}", path.display()),
            SecurityError::PathTraversalAttempt(path) => write!(f, "Path traversal attempt: {}", path.display()),
            SecurityError::FileSizeExceeded(size) => write!(f, "File size exceeded: {} bytes", size),
            SecurityError::SuspiciousArgument(arg) => write!(f, "Suspicious argument: {}", arg),
            SecurityError::ExecutionTimeout => write!(f, "Execution timeout"),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Input validation utilities
pub struct InputValidator;

impl InputValidator {
    pub fn validate_filename(filename: &str) -> Result<(), SecurityError> {
        // Prevent directory traversal
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(SecurityError::InvalidCommand("Invalid filename".to_string()));
        }

        // Prevent shell injection characters
        let dangerous_chars = [';', '&', '|', '`', '$', '(', ')', '<', '>', ' '];
        if filename.chars().any(|c| dangerous_chars.contains(&c)) {
            return Err(SecurityError::InvalidCommand("Filename contains dangerous characters".to_string()));
        }

        Ok(())
    }

    pub fn validate_url(url: &str) -> Result<(), SecurityError> {
        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(SecurityError::InvalidCommand("URL must use http or https".to_string()));
        }

        // Prevent localhost access in some security contexts
        if url.contains("localhost") || url.contains("127.0.0.1") {
            return Err(SecurityError::InvalidCommand("Localhost access not allowed".to_string()));
        }

        Ok(())
    }

    pub fn sanitize_path(path: &str) -> String {
        path.replace("..", "")
            .replace("\\", "/")
            .replace("//", "/")
            .trim_matches('/')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_policy_creation() {
        let policy = SecurityPolicy::new(SecurityLevel::Strict);
        assert_eq!(policy.level, SecurityLevel::Strict);
    }

    #[test]
    fn test_executable_whitelist() {
        let mut policy = SecurityPolicy::new(SecurityLevel::Strict);
        policy.add_allowed_executable("cargo".to_string());
        
        assert!(policy.is_executable_allowed("cargo"));
        assert!(!policy.is_executable_allowed("gcc"));
    }

    #[test]
    fn test_path_validation() {
        let mut policy = SecurityPolicy::new(SecurityLevel::Strict);
        policy.add_allowed_path(PathBuf::from("/safe"));
        
        assert!(policy.is_path_allowed(Path::new("/safe/file")));
        assert!(!policy.is_path_allowed(Path::new("/unsafe/file")));
    }

    #[test]
    fn test_file_size_limits() {
        let mut policy = SecurityPolicy::new(SecurityLevel::Strict);
        policy.set_max_file_size(1024);
        
        assert!(policy.is_file_size_allowed(512));
        assert!(!policy.is_file_size_allowed(2048));
    }

    #[test]
    fn test_suspicious_arguments() {
        let mut policy = SecurityPolicy::new(SecurityLevel::Strict);
        policy.add_allowed_executable("echo".to_string());
        policy.add_allowed_executable("rm".to_string());
        let validator = SecurityValidator::new(policy);

        // Normal arguments should pass
        assert!(validator.validate_command("echo", &["hello".to_string()]).is_ok());
        assert!(validator.validate_command("rm", &["-rf".to_string(), "/tmp/test".to_string()]).is_ok());
        // Windows paths with backslashes should pass
        assert!(validator.validate_command("echo", &["C:\\Users\\test".to_string()]).is_ok());

        // Shell injection attempts should be caught
        assert!(validator.validate_command("rm", &["-rf".to_string(), "; rm -rf /".to_string()]).is_err());
        assert!(validator.validate_command("echo", &["test; malicious".to_string()]).is_err());
        assert!(validator.validate_command("echo", &["test && malicious".to_string()]).is_err());
        assert!(validator.validate_command("echo", &["test || malicious".to_string()]).is_err());
        assert!(validator.validate_command("echo", &["test > /dev/null".to_string()]).is_err());
        assert!(validator.validate_command("echo", &["$(malicious)".to_string()]).is_err());
    }

    #[test]
    fn test_filename_validation() {
        assert!(InputValidator::validate_filename("test.txt").is_ok());
        assert!(InputValidator::validate_filename("../malicious").is_err());
        assert!(InputValidator::validate_filename("file;rm -rf /").is_err());
    }

    #[test]
    fn test_url_validation() {
        assert!(InputValidator::validate_url("https://example.com").is_ok());
        assert!(InputValidator::validate_url("http://localhost:8080").is_err());
        assert!(InputValidator::validate_url("ftp://example.com").is_err());
    }

    #[test]
    fn test_path_sanitization() {
        assert_eq!(InputValidator::sanitize_path("../safe/path"), "safe/path");
        assert_eq!(InputValidator::sanitize_path("safe//path"), "safe/path");
        assert_eq!(InputValidator::sanitize_path("\\windows\\path"), "windows/path");
    }
}
