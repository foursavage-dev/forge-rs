#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JavaBuildSystem {
    Maven,
    Gradle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaProjectConfig {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub build_system: JavaBuildSystem,
    pub skip_tests: bool,
}

impl JavaProjectConfig {
    pub fn from_maven_pom(project_dir: &Path) -> Result<Self, String> {
        let pom_path = project_dir.join("pom.xml");
        if !pom_path.exists() {
            return Err("pom.xml not found".to_string());
        }

        let content = std::fs::read_to_string(&pom_path)
            .map_err(|e| format!("Failed to read pom.xml: {}", e))?;

        // Simple XML parsing for group ID, artifact ID, and version
        let group_id =
            extract_xml_tag(&content, "groupId").unwrap_or_else(|| "com.example".to_string());
        let artifact_id =
            extract_xml_tag(&content, "artifactId").ok_or("artifactId not found in pom.xml")?;
        let version = extract_xml_tag(&content, "version").unwrap_or_else(|| "1.0.0".to_string());

        Ok(JavaProjectConfig {
            group_id,
            artifact_id,
            version,
            build_system: JavaBuildSystem::Maven,
            skip_tests: false,
        })
    }

    pub fn from_gradle_build(project_dir: &Path) -> Result<Self, String> {
        let build_path = project_dir.join("build.gradle");
        let build_kts_path = project_dir.join("build.gradle.kts");

        let (build_file, _is_kotlin) = if build_kts_path.exists() {
            (build_kts_path, true)
        } else if build_path.exists() {
            (build_path, false)
        } else {
            return Err("build.gradle or build.gradle.kts not found".to_string());
        };

        let content = std::fs::read_to_string(&build_file)
            .map_err(|e| format!("Failed to read build file: {}", e))?;

        let group_id = extract_gradle_property(&content, &["group", "grouping"])
            .unwrap_or_else(|| "com.example".to_string());
        let artifact_id =
            extract_gradle_property(&content, &["name", "artifactId", "rootProject.name"])
                .or_else(|| {
                    project_dir
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "app".to_string());
        let version =
            extract_gradle_property(&content, &["version"]).unwrap_or_else(|| "1.0.0".to_string());

        Ok(JavaProjectConfig {
            group_id,
            artifact_id,
            version,
            build_system: JavaBuildSystem::Gradle,
            skip_tests: false,
        })
    }

    pub fn detect(project_dir: &Path) -> Result<Self, String> {
        if project_dir.join("pom.xml").exists() {
            return Self::from_maven_pom(project_dir);
        }

        if project_dir.join("build.gradle").exists()
            || project_dir.join("build.gradle.kts").exists()
        {
            return Self::from_gradle_build(project_dir);
        }

        Err("No Java build system detected (pom.xml or build.gradle not found)".to_string())
    }
}

fn extract_xml_tag(content: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);

    content.find(&start_tag).and_then(|start| {
        let start = start + start_tag.len();
        content
            .find(&end_tag)
            .map(|end| content[start..end].trim().to_string())
    })
}

fn extract_gradle_property(content: &str, keys: &[&str]) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        for key in keys {
            if let Some(rest) = trimmed.strip_prefix(key) {
                let rest = rest.trim();
                let rest = rest.strip_prefix('=').unwrap_or(rest).trim();
                let rest = rest.strip_prefix('(').unwrap_or(rest).trim();
                let rest = rest.strip_suffix(')').unwrap_or(rest).trim();
                let val = rest.trim_matches(|c| c == '\'' || c == '"' || c == ' ');
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_maven_config_detection() {
        let temp = tempdir().unwrap();
        let pom_content = r#"
<?xml version="1.0" encoding="UTF-8"?>
<project>
    <groupId>com.example</groupId>
    <artifactId>test-app</artifactId>
    <version>2.0.0</version>
</project>
"#;
        std::fs::write(temp.path().join("pom.xml"), pom_content).unwrap();

        let config = JavaProjectConfig::from_maven_pom(temp.path()).unwrap();
        assert_eq!(config.group_id, "com.example");
        assert_eq!(config.artifact_id, "test-app");
        assert_eq!(config.version, "2.0.0");
        assert_eq!(config.build_system, JavaBuildSystem::Maven);
    }

    #[test]
    fn test_gradle_config_detection() {
        let temp = tempdir().unwrap();
        let build_content = r#"
plugins {
    id 'java'
}

group = 'org.test'
version = '3.0.0'
"#;
        std::fs::write(temp.path().join("build.gradle"), build_content).unwrap();

        let config = JavaProjectConfig::from_gradle_build(temp.path()).unwrap();
        assert_eq!(config.group_id, "org.test");
        assert_eq!(config.version, "3.0.0");
        assert_eq!(config.build_system, JavaBuildSystem::Gradle);
    }
}
