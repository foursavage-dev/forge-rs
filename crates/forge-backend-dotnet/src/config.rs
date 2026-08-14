#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DotnetTargetFramework {
    Net6_0,
    Net7_0,
    Net8_0,
    Net9_0,
    NetStandard2_0,
    NetStandard2_1,
    Custom(String),
}

impl DotnetTargetFramework {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "net6.0" => DotnetTargetFramework::Net6_0,
            "net7.0" => DotnetTargetFramework::Net7_0,
            "net8.0" => DotnetTargetFramework::Net8_0,
            "net9.0" => DotnetTargetFramework::Net9_0,
            "netstandard2.0" => DotnetTargetFramework::NetStandard2_0,
            "netstandard2.1" => DotnetTargetFramework::NetStandard2_1,
            custom => DotnetTargetFramework::Custom(custom.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DotnetTargetFramework::Net6_0 => "net6.0",
            DotnetTargetFramework::Net7_0 => "net7.0",
            DotnetTargetFramework::Net8_0 => "net8.0",
            DotnetTargetFramework::Net9_0 => "net9.0",
            DotnetTargetFramework::NetStandard2_0 => "netstandard2.0",
            DotnetTargetFramework::NetStandard2_1 => "netstandard2.1",
            DotnetTargetFramework::Custom(s) => s,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotnetProjectConfig {
    pub project_name: String,
    pub target_framework: DotnetTargetFramework,
    pub release: bool,
    pub run_tests: bool,
    pub publish: bool,
    pub output_path: Option<String>,
    pub runtime: Option<String>,
}

impl DotnetProjectConfig {
    pub fn from_csproj(project_dir: &Path) -> Result<Self, String> {
        let csproj_files = Self::find_csproj_files(project_dir);
        if csproj_files.is_empty() {
            return Err("No .csproj file found".to_string());
        }

        let csproj_path = &csproj_files[0];
        let content = std::fs::read_to_string(csproj_path)
            .map_err(|e| format!("Failed to read .csproj file: {}", e))?;

        let project_name = csproj_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Project")
            .to_string();

        let target_framework = Self::extract_target_framework(&content)
            .unwrap_or(DotnetTargetFramework::Net8_0);

        Ok(DotnetProjectConfig {
            project_name,
            target_framework,
            release: false,
            run_tests: true,
            publish: false,
            output_path: None,
            runtime: None,
        })
    }

    pub fn from_solution(project_dir: &Path) -> Result<Self, String> {
        let sln_files = Self::find_sln_files(project_dir);
        if sln_files.is_empty() {
            return Err("No .sln file found".to_string());
        }

        let sln_path = &sln_files[0];
        let project_name = sln_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Solution")
            .to_string();

        Ok(DotnetProjectConfig {
            project_name,
            target_framework: DotnetTargetFramework::Net8_0,
            release: false,
            run_tests: true,
            publish: false,
            output_path: None,
            runtime: None,
        })
    }

    pub fn detect(project_dir: &Path) -> Result<Self, String> {
        // Try solution file first
        if !Self::find_sln_files(project_dir).is_empty() {
            return Self::from_solution(project_dir);
        }

        // Try .csproj file
        if !Self::find_csproj_files(project_dir).is_empty() {
            return Self::from_csproj(project_dir);
        }

        Err("No .NET project files found (.sln or .csproj)".to_string())
    }

    fn find_csproj_files(project_dir: &Path) -> Vec<std::path::PathBuf> {
        let mut csproj_files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(project_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("csproj") {
                    csproj_files.push(path);
                }
            }
        }
        csproj_files
    }

    fn find_sln_files(project_dir: &Path) -> Vec<std::path::PathBuf> {
        let mut sln_files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(project_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sln") {
                    sln_files.push(path);
                }
            }
        }
        sln_files
    }

    fn extract_target_framework(content: &str) -> Option<DotnetTargetFramework> {
        // Try to extract <TargetFramework>...</TargetFramework>
        let start_tag = "<TargetFramework>";
        let end_tag = "</TargetFramework>";
        
        if let Some(start) = content.find(start_tag) {
            let start = start + start_tag.len();
            if let Some(end) = content.find(end_tag) {
                let tf_str = &content[start..end];
                return Some(DotnetTargetFramework::from_str(tf_str.trim()));
            }
        }

        // Try <TargetFrameworks> (multiple frameworks)
        let start_tag = "<TargetFrameworks>";
        let end_tag = "</TargetFrameworks>";
        
        if let Some(start) = content.find(start_tag) {
            let start = start + start_tag.len();
            if let Some(end) = content.find(end_tag) {
                let tf_str = &content[start..end];
                // Take the first framework if multiple are specified
                let first_tf = tf_str.split(';').next().unwrap_or(tf_str);
                return Some(DotnetTargetFramework::from_str(first_tf.trim()));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_csproj_config_detection() {
        let temp = tempdir().unwrap();
        let csproj_content = r#"
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>
"#;
        std::fs::write(temp.path().join("TestApp.csproj"), csproj_content).unwrap();

        let config = DotnetProjectConfig::from_csproj(temp.path()).unwrap();
        assert_eq!(config.project_name, "TestApp");
        assert_eq!(config.target_framework, DotnetTargetFramework::Net8_0);
    }

    #[test]
    fn test_target_framework_parsing() {
        assert_eq!(DotnetTargetFramework::from_str("net6.0"), DotnetTargetFramework::Net6_0);
        assert_eq!(DotnetTargetFramework::from_str("net8.0"), DotnetTargetFramework::Net8_0);
        assert_eq!(DotnetTargetFramework::from_str("custom"), DotnetTargetFramework::Custom("custom".to_string()));
    }
}
