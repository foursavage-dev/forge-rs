#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SwiftPlatform {
    IOS,
    MacOS,
    TVOS,
    WatchOS,
    Linux,
}

impl FromStr for SwiftPlatform {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ios" => Ok(SwiftPlatform::IOS),
            "macos" | "mac" => Ok(SwiftPlatform::MacOS),
            "tvos" => Ok(SwiftPlatform::TVOS),
            "watchos" => Ok(SwiftPlatform::WatchOS),
            "linux" => Ok(SwiftPlatform::Linux),
            _ => Ok(SwiftPlatform::MacOS), // Default to macOS
        }
    }
}

impl SwiftPlatform {
    pub fn as_str(&self) -> &str {
        match self {
            SwiftPlatform::IOS => "ios",
            SwiftPlatform::MacOS => "macos",
            SwiftPlatform::TVOS => "tvos",
            SwiftPlatform::WatchOS => "watchos",
            SwiftPlatform::Linux => "linux",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwiftProjectConfig {
    pub package_name: String,
    pub platform: SwiftPlatform,
    pub release: bool,
    pub run_tests: bool,
}

impl SwiftProjectConfig {
    pub fn from_package_swift(project_dir: &Path) -> Result<Self, String> {
        let package_path = project_dir.join("Package.swift");
        if !package_path.exists() {
            return Err("Package.swift not found".to_string());
        }

        let content = std::fs::read_to_string(&package_path)
            .map_err(|e| format!("Failed to read Package.swift: {}", e))?;

        // Extract package name from Package.swift
        let package_name = Self::extract_package_name(&content)
            .unwrap_or_else(|| "SwiftPackage".to_string());

        // Detect platform based on available SDKs
        let platform = Self::detect_platform();

        Ok(SwiftProjectConfig {
            package_name,
            platform,
            release: false,
            run_tests: true,
        })
    }

    pub fn from_xcode_project(project_dir: &Path) -> Result<Self, String> {
        let xcodeproj_files = Self::find_xcodeproj_files(project_dir);
        if xcodeproj_files.is_empty() {
            return Err("No .xcodeproj file found".to_string());
        }

        let xcodeproj_path = &xcodeproj_files[0];
        let project_name = xcodeproj_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("XcodeProject")
            .to_string();

        let platform = Self::detect_platform();

        Ok(SwiftProjectConfig {
            package_name: project_name,
            platform,
            release: false,
            run_tests: true,
        })
    }

    pub fn detect(project_dir: &Path) -> Result<Self, String> {
        // Try Package.swift first (Swift Package Manager)
        if project_dir.join("Package.swift").exists() {
            return Self::from_package_swift(project_dir);
        }

        // Try Xcode project
        if !Self::find_xcodeproj_files(project_dir).is_empty() {
            return Self::from_xcode_project(project_dir);
        }

        Err("No Swift project files found (Package.swift or .xcodeproj)".to_string())
    }

    fn extract_package_name(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("name:") {
                let rest = rest.trim().trim_matches(|c| c == ',' || c == ' ');
                let val = rest.trim_matches(|c| c == '\'' || c == '"' || c == ' ');
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
        None
    }

    fn detect_platform() -> SwiftPlatform {
        // Detect platform based on the current system
        #[cfg(target_os = "macos")]
        {
            return SwiftPlatform::MacOS;
        }
        #[cfg(target_os = "ios")]
        {
            return SwiftPlatform::IOS;
        }
        #[cfg(target_os = "linux")]
        {
            return SwiftPlatform::Linux;
        }
        #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "linux")))]
        {
            // Default to macOS for unknown platforms
            SwiftPlatform::MacOS
        }
    }

    fn find_xcodeproj_files(project_dir: &Path) -> Vec<std::path::PathBuf> {
        let mut xcodeproj_files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(project_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("xcodeproj") {
                    xcodeproj_files.push(path);
                }
            }
        }
        xcodeproj_files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_package_swift_config_detection() {
        let temp = tempdir().unwrap();
        let package_content = r#"
// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "TestPackage",
    products: [
        .library(name: "TestPackage", targets: ["TestPackage"]),
    ],
    targets: [
        .target(name: "TestPackage", path: "Sources"),
    ]
)
"#;
        std::fs::write(temp.path().join("Package.swift"), package_content).unwrap();

        let config = SwiftProjectConfig::from_package_swift(temp.path()).unwrap();
        assert_eq!(config.package_name, "TestPackage");
    }

    #[test]
    fn test_platform_parsing() {
        assert_eq!(SwiftPlatform::from_str("ios").unwrap(), SwiftPlatform::IOS);
        assert_eq!(SwiftPlatform::from_str("macos").unwrap(), SwiftPlatform::MacOS);
        assert_eq!(SwiftPlatform::from_str("linux").unwrap(), SwiftPlatform::Linux);
    }
}
