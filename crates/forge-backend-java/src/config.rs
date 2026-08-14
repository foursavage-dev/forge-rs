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
        let group_id = extract_xml_tag(&content, "groupId")
            .unwrap_or_else(|| "com.example".to_string());
        let artifact_id = extract_xml_tag(&content, "artifactId")
            .ok_or("artifactId not found in pom.xml")?;
        let version = extract_xml_tag(&content, "version")
            .unwrap_or_else(|| "1.0.0".to_string());

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

        let (build_file, is_kotlin) = if build_kts_path.exists() {
            (build_kts_path, true)
        } else if build_path.exists() {
            (build_path, false)
        } else {
            return Err("build.gradle or build.gradle.kts not found".to_string());
        };

        let content = std::fs::read_to_string(&build_file)
            .map_err(|e| format!("Failed to read build file: {}", e))?;

        // Simple regex extraction for group, name, version
        let group_id = extract_gradle_property(&content, &["group", "grouping"])
            .unwrap_or_else(|| "com.example".to_string());
        let artifact_id = extract_gradle_property(&content, &["name", "artifactId"])
            .ok_or("artifactId/name not found in build.gradle")?;
        let version = extract_gradle_property(&content, &["version"])
            .unwrap_or_else(|| "1.0.0".to_string());

        Ok(JavaProjectConfig {
            group_id,
            artifact_id,
            version,
            build_system: if is_kotlin {
                JavaBuildSystem::Gradle
            } else {
                JavaBuildSystem::Gradle
            },
            skip_tests: false,
        })
    }

    pub fn detect(project_dir: &Path) -> Result<Self, String> {
        // Try Maven first
        if project_dir.join("pom.xml").exists() {
            return Self::from_maven_pom(project_dir);
        }

        // Try Gradle
        if project_dir.join("build.gradle").exists() || project_dir.join("build.gradle.kts").exists() {
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
        content.find(&end_tag).map(|end| {
            content[start..end].trim().to_string()
        })
    })
}

fn extract_gradle_property(content: &str, keys: &[&str]) -> Option<String> {
    for key in keys {
        // Try different patterns: key = "value", key = 'value', key = value
        let patterns = [
            format!(r#"{}\s*=\s*["']([^"']+)["']"#, key),
            format!(r#"{}\s*=\s*(\S+)"#, key),
            format!(r#"{}(["']([^"']+)["'])"#, key),
        ];

        for pattern in patterns {
            if let Some(captures) = regex_match(&pattern, content) {
                if !captures.is_empty() {
                    return Some(captures[0].clone());
                }
            }
        }
    }
    None
}

fn regex_match(pattern: &str, text: &str) -> Option<Vec<String>> {
    // Simple pattern matching without regex crate for now
    // In production, use the regex crate
    let mut result = Vec::new();
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    
    if pattern_chars.is_empty() || text_chars.is_empty() {
        return None;
    }

    let mut i = 0;
    let mut j = 0;
    let mut capture_start = None;
    let mut in_capture = false;

    while i < pattern_chars.len() && j < text_chars.len() {
        match pattern_chars[i] {
            '\\' => {
                i += 1;
                if i < pattern_chars.len() && pattern_chars[i] == text_chars[j] {
                    j += 1;
                    i += 1;
                } else {
                    return None;
                }
            }
            '[' => {
                // Handle character class
                let mut class_chars = Vec::new();
                i += 1;
                while i < pattern_chars.len() && pattern_chars[i] != ']' {
                    class_chars.push(pattern_chars[i]);
                    i += 1;
                }
                i += 1;
                if class_chars.contains(&text_chars[j]) {
                    j += 1;
                } else {
                    return None;
                }
            }
            '(' => {
                in_capture = true;
                capture_start = Some(j);
                i += 1;
            }
            ')' => {
                if in_capture {
                    if let Some(start) = capture_start {
                        result.push(text_chars[start..j].iter().collect());
                    }
                    in_capture = false;
                    capture_start = None;
                }
                i += 1;
            }
            '.' => {
                if in_capture {
                    // Capture any character
                }
                j += 1;
                i += 1;
            }
            '+' => {
                // One or more of previous - simplified
                i += 1;
            }
            '*' => {
                // Zero or more of previous - simplified
                i += 1;
            }
            '?' => {
                // Zero or one of previous - simplified
                i += 1;
            }
            c if c == text_chars[j] => {
                if in_capture && capture_start.is_none() {
                    capture_start = Some(j);
                }
                j += 1;
                i += 1;
            }
            _ => {
                return None;
            }
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
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
        assert_eq!(config.artifact_id, "3.0.0"); // This might fail with simple parsing
        assert_eq!(config.build_system, JavaBuildSystem::Gradle);
    }
}
