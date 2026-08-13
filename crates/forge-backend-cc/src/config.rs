use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CcLanguage {
    #[default]
    C,
    Cpp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CcOutputType {
    #[default]
    Executable,
    StaticLib,
    SharedLib,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcProjectConfig {
    pub name: String,
    #[serde(default)]
    pub language: CcLanguage,
    pub sources: Vec<String>,
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default)]
    pub cflags: Vec<String>,
    #[serde(default)]
    pub cxxflags: Vec<String>,
    #[serde(default)]
    pub ldflags: Vec<String>,
    #[serde(default)]
    pub output_type: CcOutputType,
}

impl CcProjectConfig {
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn resolve_sources(&self, base_dir: &Path) -> Vec<PathBuf> {
        self.sources.iter().map(|s| base_dir.join(s)).collect()
    }

    pub fn resolve_includes(&self, base_dir: &Path) -> Vec<PathBuf> {
        self.includes.iter().map(|i| base_dir.join(i)).collect()
    }
}
