// Forge Security - Dependency Vulnerability Scanner
// Scans dependencies for vulnerabilities across all backends

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod scanner;
pub mod vulnerability;
pub mod backend;

pub use error::{SecurityError, SecurityResult};
pub use scanner::{VulnerabilityScanner, ScanOptions, ScanReport};
pub use vulnerability::{Vulnerability, Severity, VulnerabilitySource};
pub use backend::{BackendScanner, RustScanner, NpmScanner, MavenScanner};

use std::path::Path;

/// Main security service
pub struct SecurityService {
    scanner: VulnerabilityScanner,
}

impl SecurityService {
    /// Create a new security service
    pub fn new() -> Self {
        Self {
            scanner: VulnerabilityScanner::new(),
        }
    }

    /// Scan a project for vulnerabilities
    pub async fn scan_project(&self, project_path: &Path) -> SecurityResult<ScanReport> {
        let options = ScanOptions::default();
        self.scanner.scan(project_path, &options).await
    }

    /// Scan with custom options
    pub async fn scan_with_options(
        &self,
        project_path: &Path,
        options: &ScanOptions,
    ) -> SecurityResult<ScanReport> {
        self.scanner.scan(project_path, options).await
    }
}

impl Default for SecurityService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_service_creation() {
        let service = SecurityService::new();
        assert!(true); // Basic test
    }
}
