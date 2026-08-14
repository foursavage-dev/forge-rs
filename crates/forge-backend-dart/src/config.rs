#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DartProjectType {
    Dart,
    Flutter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DartTargetPlatform {
    APK,
    IOS,
    Web,
    Windows,
    MacOS,
    Linux,
    All,
}

impl DartTargetPlatform {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "apk" => DartTargetPlatform::APK,
            "ios" => DartTargetPlatform::IOS,
            "web" => DartTargetPlatform::Web,
            "windows" => DartTargetPlatform::Windows,
            "macos" => DartTargetPlatform::MacOS,
            "linux" => DartTargetPlatform::Linux,
            "all" => DartTargetPlatform::All,
            _ => DartTargetPlatform::All, // Default to all platforms
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DartTargetPlatform::APK => "apk",
            DartTargetPlatform::IOS => "ios",
            DartTargetPlatform::Web => "web",
            DartTargetPlatform::Windows => "windows",
            DartTargetPlatform::MacOS => "macos",
            DartTargetPlatform::Linux => "linux",
            DartTargetPlatform::All => "all",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DartProjectConfig {
    pub project_name: String,
    pub project_type: DartProjectType,
    pub target_platform: DartTargetPlatform,
    pub release: bool,
    pub run_tests: bool,
    pub compile: bool,
    pub is_flutter: bool,
}

impl DartProjectConfig {
    pub fn from_pubspec_yaml(project_dir: &Path) -> Result<Self, String> {
        let pubspec_path = project_dir.join("pubspec.yaml");
        if !pubspec_path.exists() {
            return Err("pubspec.yaml not found".to_string());
        }

        let content = std::fs::read_to_string(&pubspec_path)
            .map_err(|e| format!("Failed to read pubspec.yaml: {}", e))?;

        // Extract project name from pubspec.yaml
        let project_name = Self::extract_project_name(&content)
            .unwrap_or_else(|| "dart_project".to_string());

        // Detect if it's a Flutter project
        let is_flutter = Self::is_flutter_project(&content);
        let project_type = if is_flutter {
            DartProjectType::Flutter
        } else {
            DartProjectType::Dart
        };

        // Detect target platform
        let target_platform = if is_flutter {
            Self::detect_flutter_platform(&content)
        } else {
            DartTargetPlatform::All
        };

        Ok(DartProjectConfig {
            project_name,
            project_type,
            target_platform,
            release: false,
            run_tests: true,
            compile: !is_flutter, // Compile for pure Dart projects
            is_flutter,
        })
    }

    pub fn detect(project_dir: &Path) -> Result<Self, String> {
        if project_dir.join("pubspec.yaml").exists() {
            return Self::from_pubspec_yaml(project_dir);
        }

        Err("No Dart/Flutter project files found (pubspec.yaml)".to_string())
    }

    fn extract_project_name(content: &str) -> Option<String> {
        // Try to extract name: "project_name" from pubspec.yaml
        let start_marker = "name:";
        if let Some(start) = content.find(start_marker) {
            let after_start = &content[start + start_marker.len()..];
            // Skip whitespace
            let trimmed = after_start.trim_start();
            if trimmed.starts_with('"') {
                if let Some(end) = trimmed.find('"') {
                    return Some(trimmed[1..end].to_string());
                }
            } else if trimmed.starts_with('\'') {
                if let Some(end) = trimmed.find('\'') {
                    return Some(trimmed[1..end].to_string());
                }
            } else {
                // Take the first word
                if let Some(end) = trimmed.find(char::is_whitespace) {
                    return Some(trimmed[..end].to_string());
                }
            }
        }
        None
    }

    fn is_flutter_project(content: &str) -> bool {
        // Check for flutter SDK dependency
        content.contains("flutter:") || 
        content.contains("sdk: flutter") ||
        content.contains("flutter_sdk")
    }

    fn detect_flutter_platform(content: &str) -> DartTargetPlatform {
        // Try to detect target platform from pubspec.yaml
        if content.contains("android") || content.contains("android_intent") {
            return DartTargetPlatform::APK;
        }
        if content.contains("ios") || content.contains("cupertino") {
            return DartTargetPlatform::IOS;
        }
        if content.contains("web") {
            return DartTargetPlatform::Web;
        }
        if content.contains("windows") {
            return DartTargetPlatform::Windows;
        }
        if content.contains("macos") {
            return DartTargetPlatform::MacOS;
        }
        if content.contains("linux") {
            return DartTargetPlatform::Linux;
        }
        
        // Default to all platforms
        DartTargetPlatform::All
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_pubspec_config_detection() {
        let temp = tempdir().unwrap();
        let pubspec_content = r#"
name: test_app
description: A test Dart application
version: 1.0.0

environment:
  sdk: '>=3.0.0 <4.0.0'

dependencies:
  http: ^1.0.0
"#;
        std::fs::write(temp.path().join("pubspec.yaml"), pubspec_content).unwrap();

        let config = DartProjectConfig::from_pubspec_yaml(temp.path()).unwrap();
        assert_eq!(config.project_name, "test_app");
        assert_eq!(config.project_type, DartProjectType::Dart);
        assert!(!config.is_flutter);
    }

    #[test]
    fn test_flutter_project_detection() {
        let temp = tempdir().unwrap();
        let pubspec_content = r#"
name: flutter_app
description: A Flutter application
version: 1.0.0

environment:
  sdk: '>=3.0.0 <4.0.0'

dependencies:
  flutter:
    sdk: flutter
  cupertino_icons: ^1.0.0
"#;
        std::fs::write(temp.path().join("pubspec.yaml"), pubspec_content).unwrap();

        let config = DartProjectConfig::from_pubspec_yaml(temp.path()).unwrap();
        assert_eq!(config.project_name, "flutter_app");
        assert_eq!(config.project_type, DartProjectType::Flutter);
        assert!(config.is_flutter);
    }

    #[test]
    fn test_target_platform_parsing() {
        assert_eq!(DartTargetPlatform::from_str("apk"), DartTargetPlatform::APK);
        assert_eq!(DartTargetPlatform::from_str("ios"), DartTargetPlatform::IOS);
        assert_eq!(DartTargetPlatform::from_str("web"), DartTargetPlatform::Web);
    }
}
