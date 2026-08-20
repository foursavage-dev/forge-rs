use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
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
    pub race: bool,
    #[serde(default)]
    pub coverage: bool,
    #[serde(default)]
    pub run_benchmarks: bool,
    #[serde(default)]
    pub run_linter: bool,
    #[serde(default)]
    pub output_binary: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
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
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn discover_or_default(root: &Path) -> Result<Self, String> {
        let fish_go_path = root.join("fish.go.json");
        if fish_go_path.exists() {
            return Self::from_file(&fish_go_path).map_err(|e| e.to_string());
        }

        let go_mod_path = root.join("go.mod");
        let name = if go_mod_path.exists() {
            let content = fs::read_to_string(&go_mod_path).map_err(|e| e.to_string())?;
            content
                .lines()
                .find(|line| line.starts_with("module "))
                .and_then(|line| line.strip_prefix("module "))
                .map(|m| m.trim().trim_matches('"'))
                .and_then(|m| m.split('/').next_back())
                .unwrap_or("go-project")
                .to_string()
        } else {
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("go-project")
                .to_string()
        };

        Ok(Self {
            name,
            package_path: "./...".to_string(),
            tags: vec![],
            ldflags: None,
            gcflags: None,
            run_tests: true,
            race: false,
            coverage: false,
            run_benchmarks: false,
            run_linter: false,
            output_binary: None,
            env: HashMap::new(),
        })
    }
}
