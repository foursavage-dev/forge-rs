// SBOM (Software Bill of Materials) generation

use crate::error::SigningResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// SBOM format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SbomFormat {
    SPDX,
    CycloneDX,
    JSON,
}

/// SBOM metadata for signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomMetadata {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Build ID
    pub build_id: String,
    /// Dependencies
    pub dependencies: Vec<DependencyInfo>,
    /// Build tools
    pub build_tools: Vec<String>,
    /// Source commit hash
    pub source_commit: Option<String>,
    /// Build timestamp
    pub build_timestamp: DateTime<Utc>,
    /// Custom metadata
    pub custom: HashMap<String, String>,
}

impl Default for SbomMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: String::new(),
            build_id: String::new(),
            dependencies: Vec::new(),
            build_tools: Vec::new(),
            source_commit: None,
            build_timestamp: Utc::now(),
            custom: HashMap::new(),
        }
    }
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// Dependency name
    pub name: String,
    /// Dependency version
    pub version: String,
    /// License
    pub license: Option<String>,
    /// Source repository
    pub repository: Option<String>,
    /// Hash
    pub hash: Option<String>,
}

/// SBOM generator
pub struct SbomGenerator {
    format: SbomFormat,
}

impl SbomGenerator {
    /// Create a new SBOM generator
    pub fn new(format: SbomFormat) -> Self {
        Self { format }
    }

    /// Generate SBOM for a package
    pub async fn generate(&self, package_path: &Path) -> SigningResult<String> {
        let metadata = self.extract_metadata(package_path).await?;

        match self.format {
            SbomFormat::SPDX => self.generate_spdx(&metadata),
            SbomFormat::CycloneDX => self.generate_cyclonedx(&metadata),
            SbomFormat::JSON => self.generate_json(&metadata),
        }
    }

    /// Extract metadata from package
    async fn extract_metadata(&self, package_path: &Path) -> SigningResult<SbomMetadata> {
        // This would parse actual package files (Cargo.toml, package.json, etc.)
        // For now, return basic metadata
        let name = package_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(SbomMetadata {
            name,
            version: "0.1.0".to_string(),
            build_id: uuid::Uuid::new_v4().to_string(),
            build_timestamp: Utc::now(),
            ..Default::default()
        })
    }

    /// Generate SPDX format SBOM
    fn generate_spdx(&self, metadata: &SbomMetadata) -> SigningResult<String> {
        let mut sbom = String::new();
        sbom.push_str("SPDXVersion: SPDX-2.3\n");
        sbom.push_str(&format!("PackageName: {}\n", metadata.name));
        sbom.push_str(&format!("PackageVersion: {}\n", metadata.version));
        sbom.push_str(&format!("BuildID: {}\n", metadata.build_id));
        sbom.push_str(&format!("BuildTimestamp: {}\n", metadata.build_timestamp.to_rfc3339()));

        for dep in &metadata.dependencies {
            sbom.push_str(&format!("Dependency: {}@{}\n", dep.name, dep.version));
        }

        Ok(sbom)
    }

    /// Generate CycloneDX format SBOM
    fn generate_cyclonedx(&self, metadata: &SbomMetadata) -> SigningResult<String> {
        let components: Vec<serde_json::Value> = metadata.dependencies.iter().map(|dep| {
            let mut comp = serde_json::json!({
                "name": dep.name,
                "version": dep.version,
                "type": "library"
            });

            if let Some(license) = &dep.license {
                comp["licenses"] = serde_json::json!([{"license": {"id": license}}]);
            }

            if let Some(repo) = &dep.repository {
                comp["externalReferences"] = serde_json::json!([{
                    "type": "vcs",
                    "url": repo
                }]);
            }

            comp
        }).collect();

        let bom = serde_json::json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.5",
            "metadata": {
                "component": {
                    "name": metadata.name,
                    "version": metadata.version,
                    "type": "application"
                },
                "properties": [
                    {"name": "build_id", "value": metadata.build_id},
                    {"name": "build_timestamp", "value": metadata.build_timestamp.to_rfc3339()}
                ]
            },
            "components": components
        });

        serde_json::to_string_pretty(&bom).map_err(Into::into)
    }

    /// Generate JSON format SBOM
    fn generate_json(&self, metadata: &SbomMetadata) -> SigningResult<String> {
        serde_json::to_string_pretty(metadata).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sbom_generation() {
        let generator = SbomGenerator::new(SbomFormat::JSON);
        let temp_dir = tempfile::tempdir().unwrap();
        let result = generator.generate(temp_dir.path()).await.unwrap();
        assert!(result.contains("name"));
    }

    #[tokio::test]
    async fn test_spdx_generation() {
        let generator = SbomGenerator::new(SbomFormat::SPDX);
        let temp_dir = tempfile::tempdir().unwrap();
        let result = generator.generate(temp_dir.path()).await.unwrap();
        assert!(result.contains("SPDXVersion"));
    }
}
