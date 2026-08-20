use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolchainKind {
    Rust,
    Node,
    Go,
    Cpp,
    Python,
    Java,
    Dotnet,
    Docker,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainSpec {
    pub kind: ToolchainKind,
    pub version: String,
    pub path: PathBuf,
    pub envs: HashMap<String, String>,
    pub checksum: Option<String>,
    pub is_hermetic: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ToolchainRegistry {
    toolchains: HashMap<ToolchainKind, ToolchainSpec>,
}

fn find_executable_in_path(names: &[&str]) -> Option<PathBuf> {
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            for name in names {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
                #[cfg(windows)]
                {
                    let candidate_exe = dir.join(format!("{}.exe", name));
                    if candidate_exe.is_file() {
                        return Some(candidate_exe);
                    }
                }
            }
        }
    }
    None
}

impl ToolchainRegistry {
    pub fn new() -> Self {
        Self {
            toolchains: HashMap::new(),
        }
    }

    pub fn register(&mut self, spec: ToolchainSpec) {
        self.toolchains.insert(spec.kind.clone(), spec);
    }

    pub fn get(&self, kind: &ToolchainKind) -> Option<&ToolchainSpec> {
        self.toolchains.get(kind)
    }

    pub fn has_hermetic(&self, kind: &ToolchainKind) -> bool {
        self.toolchains
            .get(kind)
            .is_some_and(|spec| spec.is_hermetic)
    }

    pub fn auto_detect_system() -> Self {
        let mut registry = Self::new();

        if let Some(path) = find_executable_in_path(&["cargo", "rustc"]) {
            registry.register(ToolchainSpec {
                kind: ToolchainKind::Rust,
                version: "system".to_string(),
                path,
                envs: HashMap::new(),
                checksum: None,
                is_hermetic: false,
            });
        }

        if let Some(path) = find_executable_in_path(&["node", "npm", "pnpm", "yarn"]) {
            registry.register(ToolchainSpec {
                kind: ToolchainKind::Node,
                version: "system".to_string(),
                path,
                envs: HashMap::new(),
                checksum: None,
                is_hermetic: false,
            });
        }

        if let Some(path) = find_executable_in_path(&["go"]) {
            registry.register(ToolchainSpec {
                kind: ToolchainKind::Go,
                version: "system".to_string(),
                path,
                envs: HashMap::new(),
                checksum: None,
                is_hermetic: false,
            });
        }

        if let Some(path) = find_executable_in_path(&["clang", "gcc", "cl"]) {
            registry.register(ToolchainSpec {
                kind: ToolchainKind::Cpp,
                version: "system".to_string(),
                path,
                envs: HashMap::new(),
                checksum: None,
                is_hermetic: false,
            });
        }

        if let Some(path) = find_executable_in_path(&["python3", "python"]) {
            registry.register(ToolchainSpec {
                kind: ToolchainKind::Python,
                version: "system".to_string(),
                path,
                envs: HashMap::new(),
                checksum: None,
                is_hermetic: false,
            });
        }

        registry
    }

    pub fn configure_hermetic<P: AsRef<Path>>(
        &mut self,
        kind: ToolchainKind,
        version: &str,
        isolated_path: P,
    ) -> Result<()> {
        let path = isolated_path.as_ref().to_path_buf();
        let mut envs = HashMap::new();

        if let Some(parent) = path.parent() {
            envs.insert("PATH".to_string(), parent.to_string_lossy().to_string());
        }

        self.register(ToolchainSpec {
            kind,
            version: version.to_string(),
            path,
            envs,
            checksum: None,
            is_hermetic: true,
        });

        Ok(())
    }

    pub fn apply_to_command(
        &self,
        kind: &ToolchainKind,
        cmd: &mut std::process::Command,
    ) -> Result<()> {
        if let Some(spec) = self.get(kind) {
            for (key, val) in &spec.envs {
                cmd.env(key, val);
            }
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.toolchains.len()
    }

    pub fn is_empty(&self) -> bool {
        self.toolchains.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolchain_registry() {
        let mut registry = ToolchainRegistry::new();
        assert!(registry.is_empty());

        registry
            .configure_hermetic(
                ToolchainKind::Rust,
                "1.88.0",
                PathBuf::from("/hermetic/bin/cargo"),
            )
            .unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry.has_hermetic(&ToolchainKind::Rust));
        assert!(!registry.has_hermetic(&ToolchainKind::Go));

        let spec = registry.get(&ToolchainKind::Rust).unwrap();
        assert_eq!(spec.version, "1.88.0");
        assert_eq!(spec.path, PathBuf::from("/hermetic/bin/cargo"));
    }

    #[test]
    fn test_auto_detect() {
        let registry = ToolchainRegistry::auto_detect_system();
        assert!(!registry.is_empty());
    }
}
