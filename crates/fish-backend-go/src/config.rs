use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoProjectConfig {
    pub name: String,
    #[serde(default = "default_package_path")]
    pub package_path: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub ldflags: Option<String>,
    #[serde(default)]
    pub gcflags: Option<String>,
    #[serde(default = "default_run_tests")]
    pub run_tests: bool,
    #[serde(default)]
    pub output_binary: Option<String>,
}

fn default_package_path() -> String {
    "./...".to_string()
}

fn default_run_tests() -> bool {
    true
}

impl GoProjectConfig {
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
