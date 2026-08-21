use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PythonRunner {
    #[default]
    Uv,
    Python,
    Poetry,
}

impl PythonRunner {
    pub fn executable(&self) -> &'static str {
        match self {
            PythonRunner::Uv => "uv",
            PythonRunner::Python => {
                if cfg!(windows) {
                    "python"
                } else {
                    "python3"
                }
            }
            PythonRunner::Poetry => "poetry",
        }
    }

    pub fn detect(root: &Path) -> Self {
        if root.join("uv.lock").exists() {
            PythonRunner::Uv
        } else if root.join("poetry.lock").exists() {
            PythonRunner::Poetry
        } else {
            PythonRunner::Uv
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PackagingType {
    #[default]
    Standard,
    Pex,
    Wheel,
    Sdist,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PexConfig {
    #[serde(default)]
    pub entry_point: Option<String>,
    #[serde(default)]
    pub output_pex: Option<String>,
    #[serde(default)]
    pub interpreter_constraint: Option<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub inherit_path: Option<String>,
    #[serde(default)]
    pub include_tools: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyTaskSpec {
    pub name: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyProjectConfig {
    pub name: String,
    #[serde(default)]
    pub runner: Option<PythonRunner>,
    #[serde(default)]
    pub packaging: Option<PackagingType>,
    #[serde(default)]
    pub pex: Option<PexConfig>,
    #[serde(default)]
    pub tasks: Vec<PyTaskSpec>,
    #[serde(default)]
    pub source_dirs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PyProjectToml {
    project: Option<PyProjectMetadata>,
}

#[derive(Debug, Deserialize)]
struct PyProjectMetadata {
    name: Option<String>,
}

impl PyProjectConfig {
    pub fn from_fish_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn discover_or_default(root: &Path) -> Result<Self, String> {
        let fish_py_path = root.join("fish.py.json");
        if fish_py_path.exists() {
            return Self::from_fish_file(&fish_py_path);
        }
        let forge_py_path = root.join("forge.py.json");
        if forge_py_path.exists() {
            return Self::from_fish_file(&forge_py_path);
        }

        let pyproject_path = root.join("pyproject.toml");
        let name = if pyproject_path.exists() {
            let content = fs::read_to_string(&pyproject_path).map_err(|e| e.to_string())?;
            let parsed: Result<PyProjectToml, _> = toml::from_str(&content);
            parsed
                .ok()
                .and_then(|p| p.project)
                .and_then(|p| p.name)
                .unwrap_or_else(|| {
                    root.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("py-project")
                        .to_string()
                })
        } else if root.join("setup.py").exists() || root.join("requirements.txt").exists() {
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("py-project")
                .to_string()
        } else {
            return Err("no pyproject.toml, setup.py, or requirements.txt found".to_string());
        };

        let runner = PythonRunner::detect(root);
        let tasks = vec![
            PyTaskSpec {
                name: "lint".to_string(),
                command: Some("ruff".to_string()),
                args: vec!["check".to_string(), ".".to_string()],
                depends_on: vec![],
            },
            PyTaskSpec {
                name: "typecheck".to_string(),
                command: Some("mypy".to_string()),
                args: vec![".".to_string()],
                depends_on: vec!["lint".to_string()],
            },
            PyTaskSpec {
                name: "test".to_string(),
                command: Some("pytest".to_string()),
                args: vec![],
                depends_on: vec!["typecheck".to_string()],
            },
            PyTaskSpec {
                name: "build".to_string(),
                command: Some("uv".to_string()),
                args: vec!["build".to_string()],
                depends_on: vec!["test".to_string()],
            },
        ];

        Ok(Self {
            name,
            runner: Some(runner),
            packaging: Some(PackagingType::Standard),
            pex: None,
            tasks,
            source_dirs: vec!["src".to_string()],
        })
    }
}
