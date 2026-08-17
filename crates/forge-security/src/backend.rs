use crate::error::{SecurityError, SecurityResult};
use crate::scanner::ScanOptions;
use crate::vulnerability::{Severity, Vulnerability, VulnerabilitySource};
use semver::Version;
use std::path::Path;

pub trait BackendScanner {
    fn scan(
        &self,
        project_path: &Path,
        options: &ScanOptions,
    ) -> impl std::future::Future<Output = SecurityResult<Vec<Vulnerability>>> + Send;
}

#[derive(Clone, Default)]
pub struct RustScanner;

impl RustScanner {
    pub fn new() -> Self {
        Self
    }

    fn parse_cargo_lock(content: &str) -> Vec<(String, String)> {
        let mut packages = Vec::new();
        let mut current_name: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name = ") {
                let name = line.trim_start_matches("name = ").trim_matches('"');
                current_name = Some(name.to_string());
            } else if line.starts_with("version = ") {
                if let Some(name) = current_name.take() {
                    let version = line.trim_start_matches("version = ").trim_matches('"');
                    packages.push((name, version.to_string()));
                }
            } else if line == "[[package]]" {
                current_name = None;
            }
        }

        packages
    }

    fn check_advisories(packages: &[(String, String)]) -> Vec<Vulnerability> {
        let known_rules: &[(&str, &str, &str, Severity, &str)] = &[
            (
                "lru",
                "0.16.3",
                "GHSA-rhfx-m35p-ff5j",
                Severity::Low,
                "IterMut violates Stacked Borrows by invalidating internal pointer",
            ),
            (
                "h2",
                "0.3.26",
                "RUSTSEC-2024-0332",
                Severity::High,
                "HTTP/2 CONTINUATION flood can lead to unbounded CPU allocation",
            ),
            (
                "hyper",
                "0.14.30",
                "RUSTSEC-2024-0376",
                Severity::Medium,
                "HTTP request smuggling via chunk extension parsing",
            ),
            (
                "openssl",
                "0.10.60",
                "RUSTSEC-2023-0052",
                Severity::High,
                "OpenSSL memory corruption in legacy bindings",
            ),
            (
                "tar",
                "0.4.40",
                "RUSTSEC-2023-0001",
                Severity::Medium,
                "Directory traversal through malformed archive entries",
            ),
            (
                "rsa",
                "0.9.6",
                "RUSTSEC-2023-0071",
                Severity::High,
                "Marvin attack side-channel leakage in PKCS#1 v1.5 decryption",
            ),
        ];

        let mut results = Vec::new();
        for (pkg_name, pkg_ver) in packages {
            if let Ok(ver) = Version::parse(pkg_ver) {
                for (name, fix_ver_str, id, severity, desc) in known_rules {
                    if pkg_name == name && Version::parse(fix_ver_str).is_ok_and(|fix_ver| ver < fix_ver) {
                        let mut vuln = Vulnerability::new(
                            id.to_string(),
                            pkg_name.clone(),
                            *severity,
                        );
                        vuln.source = VulnerabilitySource::RustSec;
                        vuln.description = desc.to_string();
                        vuln.affected_versions = format!("< {fix_ver_str}");
                        vuln.fixed_version = Some(fix_ver_str.to_string());
                        results.push(vuln);
                    }
                }
            }
        }
        results
    }
}

impl BackendScanner for RustScanner {
    async fn scan(
        &self,
        project_path: &Path,
        _options: &ScanOptions,
    ) -> SecurityResult<Vec<Vulnerability>> {
        let lock_file = project_path.join("Cargo.lock");
        if !lock_file.exists() {
            return Err(SecurityError::LockFileNotFound(
                "Cargo.lock not found".to_string(),
            ));
        }

        let content = tokio::fs::read_to_string(&lock_file)
            .await
            .map_err(SecurityError::IoError)?;

        let packages = Self::parse_cargo_lock(&content);
        Ok(Self::check_advisories(&packages))
    }
}

#[derive(Clone, Default)]
pub struct NpmScanner;

impl NpmScanner {
    pub fn new() -> Self {
        Self
    }

    fn check_advisories(packages: &[(String, String)]) -> Vec<Vulnerability> {
        let known_rules: &[(&str, &str, &str, Severity, &str)] = &[
            (
                "lodash",
                "4.17.21",
                "CVE-2021-23337",
                Severity::High,
                "Command injection in template function",
            ),
            (
                "axios",
                "1.7.4",
                "CVE-2024-39338",
                Severity::Medium,
                "Server-side request forgery (SSRF) bypass",
            ),
            (
                "tar",
                "6.2.1",
                "CVE-2024-28863",
                Severity::High,
                "Denial of service via unbounded memory allocation",
            ),
            (
                "express",
                "4.19.2",
                "CVE-2024-29041",
                Severity::Medium,
                "Open redirect vulnerability in res.location",
            ),
            (
                "ws",
                "8.17.1",
                "CVE-2024-37890",
                Severity::High,
                "Resource exhaustion via crafted Sec-WebSocket-Extensions",
            ),
        ];

        let mut results = Vec::new();
        for (pkg_name, pkg_ver) in packages {
            if let Ok(ver) = Version::parse(pkg_ver) {
                for (name, fix_ver_str, id, severity, desc) in known_rules {
                    if pkg_name == name && Version::parse(fix_ver_str).is_ok_and(|fix_ver| ver < fix_ver) {
                        let mut vuln = Vulnerability::new(
                            id.to_string(),
                            pkg_name.clone(),
                            *severity,
                        );
                        vuln.source = VulnerabilitySource::NPM;
                        vuln.description = desc.to_string();
                        vuln.affected_versions = format!("< {fix_ver_str}");
                        vuln.fixed_version = Some(fix_ver_str.to_string());
                        results.push(vuln);
                    }
                }
            }
        }
        results
    }
}

impl BackendScanner for NpmScanner {
    async fn scan(
        &self,
        project_path: &Path,
        _options: &ScanOptions,
    ) -> SecurityResult<Vec<Vulnerability>> {
        let lock_file = project_path.join("package-lock.json");
        if !lock_file.exists() {
            return Err(SecurityError::LockFileNotFound(
                "package-lock.json not found".to_string(),
            ));
        }

        let content = tokio::fs::read_to_string(&lock_file)
            .await
            .map_err(SecurityError::IoError)?;

        let mut packages = Vec::new();
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(deps) = val.get("dependencies").and_then(|d| d.as_object()) {
                for (name, info) in deps {
                    if let Some(ver) = info.get("version").and_then(|v| v.as_str()) {
                        packages.push((name.clone(), ver.to_string()));
                    }
                }
            }
            if let Some(pkgs) = val.get("packages").and_then(|p| p.as_object()) {
                for (path, info) in pkgs {
                    let name = path.trim_start_matches("node_modules/").to_string();
                    if let Some(ver) = info.get("version").and_then(|v| v.as_str()).filter(|_| !name.is_empty()) {
                        packages.push((name, ver.to_string()));
                    }
                }
            }
        }

        Ok(Self::check_advisories(&packages))
    }
}

#[derive(Clone, Default)]
pub struct MavenScanner;

impl MavenScanner {
    pub fn new() -> Self {
        Self
    }
}

impl BackendScanner for MavenScanner {
    async fn scan(
        &self,
        project_path: &Path,
        _options: &ScanOptions,
    ) -> SecurityResult<Vec<Vulnerability>> {
        let pom_file = project_path.join("pom.xml");
        if !pom_file.exists() {
            return Err(SecurityError::LockFileNotFound(
                "pom.xml not found".to_string(),
            ));
        }

        let content = tokio::fs::read_to_string(&pom_file)
            .await
            .map_err(SecurityError::IoError)?;

        let mut results = Vec::new();
        if content.contains("log4j-core") && (content.contains("2.14.") || content.contains("2.15.")) {
            let mut vuln = Vulnerability::new(
                "CVE-2021-44228".to_string(),
                "log4j-core".to_string(),
                Severity::Critical,
            );
            vuln.source = VulnerabilitySource::Maven;
            vuln.description = "Remote code execution via JNDI lookup (Log4Shell)".to_string();
            vuln.fixed_version = Some("2.17.1".to_string());
            results.push(vuln);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rust_scanner_missing_file() {
        let scanner = RustScanner::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let result = scanner.scan(temp_dir.path(), &ScanOptions::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rust_scanner_detects_vulnerable_package() {
        let scanner = RustScanner::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let lock_path = temp_dir.path().join("Cargo.lock");
        let lock_content = r#"
version = 3

[[package]]
name = "lru"
version = "0.12.5"

[[package]]
name = "serde"
version = "1.0.219"
"#;
        tokio::fs::write(&lock_path, lock_content).await.unwrap();

        let report = scanner.scan(temp_dir.path(), &ScanOptions::default()).await.unwrap();
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].package, "lru");
        assert_eq!(report[0].id, "GHSA-rhfx-m35p-ff5j");
        assert_eq!(report[0].severity, Severity::Low);
    }

    #[tokio::test]
    async fn test_rust_scanner_clean_project() {
        let scanner = RustScanner::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let lock_path = temp_dir.path().join("Cargo.lock");
        let lock_content = r#"
version = 3

[[package]]
name = "lru"
version = "0.18.2"

[[package]]
name = "ratatui"
version = "0.30.2"
"#;
        tokio::fs::write(&lock_path, lock_content).await.unwrap();

        let report = scanner.scan(temp_dir.path(), &ScanOptions::default()).await.unwrap();
        assert_eq!(report.len(), 0);
    }
}
