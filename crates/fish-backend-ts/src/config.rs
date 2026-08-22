use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    #[default]
    Npm,
    Pnpm,
    Yarn,
    Bun,
}

impl PackageManager {
    pub fn executable(&self) -> &'static str {
        if cfg!(windows) {
            match self {
                PackageManager::Npm => "npm.cmd",
                PackageManager::Pnpm => "pnpm.cmd",
                PackageManager::Yarn => "yarn.cmd",
                PackageManager::Bun => "bun.exe",
            }
        } else {
            match self {
                PackageManager::Npm => "npm",
                PackageManager::Pnpm => "pnpm",
                PackageManager::Yarn => "yarn",
                PackageManager::Bun => "bun",
            }
        }
    }

    pub fn detect(root: &Path) -> Self {
        if root.join("pnpm-lock.yaml").exists() {
            PackageManager::Pnpm
        } else if root.join("bun.lockb").exists() || root.join("bun.lock").exists() {
            PackageManager::Bun
        } else if root.join("yarn.lock").exists() {
            PackageManager::Yarn
        } else {
            PackageManager::Npm
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsTaskSpec {
    pub name: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsProjectConfig {
    pub name: String,
    #[serde(default)]
    pub package_manager: Option<PackageManager>,
    #[serde(default)]
    pub tasks: Vec<TsTaskSpec>,
    #[serde(default)]
    pub source_dirs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    name: Option<String>,
    #[serde(default)]
    scripts: std::collections::BTreeMap<String, String>,
}

impl TsProjectConfig {
    pub fn from_fish_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn discover_or_default(root: &Path) -> Result<Self, String> {
        let fish_ts_path = root.join("fish.ts.json");
        if fish_ts_path.exists() {
            return Self::from_fish_file(&fish_ts_path);
        }

        let pkg_path = root.join("package.json");
        if !pkg_path.exists() {
            return Err("neither fish.ts.json nor package.json found".to_string());
        }

        let content = fs::read_to_string(&pkg_path).map_err(|e| e.to_string())?;
        let pkg: PackageJson = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        let name = pkg.name.unwrap_or_else(|| {
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("ts-project")
                .to_string()
        });

        let pm = PackageManager::detect(root);
        let mut tasks = Vec::new();

        if pkg.scripts.contains_key("typecheck") {
            tasks.push(TsTaskSpec {
                name: "typecheck".to_string(),
                command: None,
                args: vec!["run".to_string(), "typecheck".to_string()],
                depends_on: vec![],
            });
        } else if root.join("tsconfig.json").exists() {
            let npx_cmd = if cfg!(windows) { "npx.cmd" } else { "npx" };
            tasks.push(TsTaskSpec {
                name: "typecheck".to_string(),
                command: Some(npx_cmd.to_string()),
                args: vec!["tsc".to_string(), "--noEmit".to_string()],
                depends_on: vec![],
            });
        }

        if pkg.scripts.contains_key("build") {
            let depends = if tasks.iter().any(|t| t.name == "typecheck") {
                vec!["typecheck".to_string()]
            } else {
                vec![]
            };
            tasks.push(TsTaskSpec {
                name: "build".to_string(),
                command: None,
                args: vec!["run".to_string(), "build".to_string()],
                depends_on: depends,
            });
        }

        if pkg.scripts.contains_key("test") {
            let depends = if tasks.iter().any(|t| t.name == "build") {
                vec!["build".to_string()]
            } else if tasks.iter().any(|t| t.name == "typecheck") {
                vec!["typecheck".to_string()]
            } else {
                vec![]
            };
            tasks.push(TsTaskSpec {
                name: "test".to_string(),
                command: None,
                args: vec!["run".to_string(), "test".to_string()],
                depends_on: depends,
            });
        }

        if tasks.is_empty() {
            tasks.push(TsTaskSpec {
                name: "build".to_string(),
                command: None,
                args: vec!["run".to_string(), "build".to_string()],
                depends_on: vec![],
            });
        }

        Ok(Self {
            name,
            package_manager: Some(pm),
            tasks,
            source_dirs: vec!["src".to_string()],
        })
    }
}
