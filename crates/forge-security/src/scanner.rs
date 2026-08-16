// Main vulnerability scanner

use crate::backend::{BackendScanner, MavenScanner, NpmScanner, RustScanner};
use crate::error::SecurityResult;
use crate::vulnerability::{Severity, Vulnerability};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Scan options
#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    /// Minimum severity to report
    pub min_severity: Severity,
    /// Whether to block builds on vulnerabilities
    pub block_on_vulnerabilities: bool,
    /// Whether to scan dev dependencies
    pub scan_dev_dependencies: bool,
    /// Maximum number of vulnerabilities to return
    pub max_results: Option<usize>,
}

/// Scan report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    /// Project path
    pub project_path: String,
    /// Scan timestamp
    pub scan_timestamp: DateTime<Utc>,
    /// Total vulnerabilities found
    pub total_vulnerabilities: usize,
    /// Vulnerabilities by severity
    pub by_severity: HashMap<Severity, usize>,
    /// Vulnerabilities found
    pub vulnerabilities: Vec<Vulnerability>,
    /// Scan duration in seconds
    pub scan_duration_secs: f64,
    /// Whether scan should block build
    pub should_block: bool,
}

/// Main vulnerability scanner
pub struct VulnerabilityScanner {
    rust_scanner: RustScanner,
    npm_scanner: NpmScanner,
    maven_scanner: MavenScanner,
}

impl VulnerabilityScanner {
    /// Create a new scanner
    pub fn new() -> Self {
        Self {
            rust_scanner: RustScanner::new(),
            npm_scanner: NpmScanner::new(),
            maven_scanner: MavenScanner::new(),
        }
    }

    /// Scan a project for vulnerabilities
    pub async fn scan(
        &self,
        project_path: &Path,
        options: &ScanOptions,
    ) -> SecurityResult<ScanReport> {
        let start_time = std::time::Instant::now();
        let mut all_vulnerabilities = Vec::new();

        // Try each backend scanner
        if let Ok(rust_vulns) = self.rust_scanner.scan(project_path, options).await {
            all_vulnerabilities.extend(rust_vulns);
        }

        if let Ok(npm_vulns) = self.npm_scanner.scan(project_path, options).await {
            all_vulnerabilities.extend(npm_vulns);
        }

        if let Ok(maven_vulns) = self.maven_scanner.scan(project_path, options).await {
            all_vulnerabilities.extend(maven_vulns);
        }

        // Filter by severity
        let vulnerabilities: Vec<Vulnerability> = all_vulnerabilities
            .into_iter()
            .filter(|v| v.severity >= options.min_severity)
            .collect();

        // Limit results if specified
        let vulnerabilities = if let Some(max) = options.max_results {
            vulnerabilities.into_iter().take(max).collect()
        } else {
            vulnerabilities
        };

        // Count by severity
        let mut by_severity = HashMap::new();
        for vuln in &vulnerabilities {
            *by_severity.entry(vuln.severity).or_insert(0) += 1;
        }

        let should_block = options.block_on_vulnerabilities
            && vulnerabilities
                .iter()
                .any(|v| v.severity >= options.min_severity);

        let scan_duration = start_time.elapsed().as_secs_f64();

        Ok(ScanReport {
            project_path: project_path.display().to_string(),
            scan_timestamp: Utc::now(),
            total_vulnerabilities: vulnerabilities.len(),
            by_severity,
            vulnerabilities,
            scan_duration_secs: scan_duration,
            should_block,
        })
    }
}

impl Default for VulnerabilityScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scanner_creation() {
        let _scanner = VulnerabilityScanner::new();
        assert!(true);
    }
}
