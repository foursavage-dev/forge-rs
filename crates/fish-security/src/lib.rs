// Fish Security - Dependency Vulnerability Scanner
// Scans dependencies for vulnerabilities across all backends

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod backend;
pub mod error;
pub mod scanner;
pub mod vulnerability;

pub use backend::{BackendScanner, MavenScanner, NpmScanner, RustScanner};
pub use error::{SecurityError, SecurityResult};
pub use scanner::{ScanOptions, ScanReport, VulnerabilityScanner};
pub use vulnerability::{Severity, Vulnerability, VulnerabilitySource};

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
        let _service = SecurityService::new();
        // Security service creation test - no assertion needed
    }
}
