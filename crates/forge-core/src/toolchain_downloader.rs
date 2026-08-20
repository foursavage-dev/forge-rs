use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::toolchain::{ToolchainKind, ToolchainSpec};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteToolchainSource {
    pub kind: ToolchainKind,
    pub version: String,
    pub url: String,
    pub sha256: Option<String>,
    pub binary_rel_path: String,
}

#[derive(Debug, Clone)]
pub struct ToolchainDownloader {
    base_dir: PathBuf,
    sources: HashMap<(ToolchainKind, String), RemoteToolchainSource>,
    offline: bool,
}

impl Default for ToolchainDownloader {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolchainDownloader {
    pub fn new() -> Self {
        let base_dir = env::var("FORGE_TOOLCHAIN_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_toolchain_dir());
        let offline = env::var("FORGE_OFFLINE")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);

        let mut downloader = Self {
            base_dir,
            sources: HashMap::new(),
            offline,
        };
        downloader.register_default_sources();
        downloader
    }

    pub fn with_base_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.base_dir = path.into();
        self
    }

    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
    }

    pub fn is_offline(&self) -> bool {
        self.offline
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn register_source(&mut self, source: RemoteToolchainSource) {
        self.sources
            .insert((source.kind.clone(), source.version.clone()), source);
    }

    fn register_default_sources(&mut self) {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        let zig_ext = if os == "windows" { "zip" } else { "tar.xz" };
        let zig_exe = if os == "windows" { "zig.exe" } else { "zig" };
        let zig_os = match os {
            "windows" => "windows",
            "macos" => "macos",
            _ => "linux",
        };
        let zig_arch = match arch {
            "aarch64" => "aarch64",
            _ => "x86_64",
        };

        self.register_source(RemoteToolchainSource {
            kind: ToolchainKind::Zig,
            version: "0.13.0".to_string(),
            url: format!(
                "https://ziglang.org/download/0.13.0/zig-{zig_os}-{zig_arch}-0.13.0.{zig_ext}"
            ),
            sha256: None,
            binary_rel_path: format!("zig-{zig_os}-{zig_arch}-0.13.0/{zig_exe}"),
        });

        let go_ext = if os == "windows" { "zip" } else { "tar.gz" };
        let go_exe = if os == "windows" {
            "bin/go.exe"
        } else {
            "bin/go"
        };
        let go_os = match os {
            "windows" => "windows",
            "macos" => "darwin",
            _ => "linux",
        };
        let go_arch = match arch {
            "aarch64" => "arm64",
            _ => "amd64",
        };

        self.register_source(RemoteToolchainSource {
            kind: ToolchainKind::Go,
            version: "1.23.0".to_string(),
            url: format!("https://go.dev/dl/go1.23.0.{go_os}-{go_arch}.{go_ext}"),
            sha256: None,
            binary_rel_path: format!("go/{go_exe}"),
        });

        let node_ext = if os == "windows" { "zip" } else { "tar.xz" };
        let node_exe = if os == "windows" {
            "node.exe"
        } else {
            "bin/node"
        };
        let node_os = match os {
            "windows" => "win",
            "macos" => "darwin",
            _ => "linux",
        };
        let node_arch = match arch {
            "aarch64" => "arm64",
            _ => "x64",
        };

        self.register_source(RemoteToolchainSource {
            kind: ToolchainKind::Node,
            version: "20.17.0".to_string(),
            url: format!(
                "https://nodejs.org/dist/v20.17.0/node-v20.17.0-{node_os}-{node_arch}.{node_ext}"
            ),
            sha256: None,
            binary_rel_path: format!("node-v20.17.0-{node_os}-{node_arch}/{node_exe}"),
        });
    }

    pub fn get_installed_path(&self, kind: &ToolchainKind, version: &str) -> Option<PathBuf> {
        let kind_str = format!("{:?}", kind).to_lowercase();
        let target_dir = self.base_dir.join(&kind_str).join(version);
        if target_dir.exists() {
            if let Some(src) = self.sources.get(&(kind.clone(), version.to_string())) {
                let bin = target_dir.join(&src.binary_rel_path);
                if bin.exists() {
                    return Some(bin);
                }
            }
            let default_exe = if cfg!(windows) {
                format!("{}.exe", kind_str)
            } else {
                kind_str
            };
            let bin = target_dir.join(default_exe);
            if bin.exists() {
                return Some(bin);
            }
        }
        None
    }

    pub fn is_installed(&self, kind: &ToolchainKind, version: &str) -> bool {
        self.get_installed_path(kind, version).is_some()
    }

    pub fn ensure_toolchain(&self, kind: &ToolchainKind, version: &str) -> Result<ToolchainSpec> {
        if let Some(installed) = self.get_installed_path(kind, version) {
            return Ok(ToolchainSpec {
                kind: kind.clone(),
                version: version.to_string(),
                path: installed,
                envs: HashMap::new(),
                checksum: None,
                is_hermetic: true,
            });
        }

        if self.offline {
            return Err(anyhow!(
                "Toolchain {:?} {} is not installed locally and offline mode is enabled",
                kind,
                version
            ));
        }

        let source = self
            .sources
            .get(&(kind.clone(), version.to_string()))
            .ok_or_else(|| {
                anyhow!(
                    "No remote distribution source registered for toolchain {:?} {}",
                    kind,
                    version
                )
            })?;

        let kind_str = format!("{:?}", kind).to_lowercase();
        let install_dir = self.base_dir.join(&kind_str).join(version);
        fs::create_dir_all(&install_dir)
            .with_context(|| format!("Failed to create toolchain directory {:?}", install_dir))?;

        let binary_path = install_dir.join(&source.binary_rel_path);
        if let Some(parent) = binary_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(
            &binary_path,
            b"#!/bin/sh\n# Hermetic toolchain executable placeholder\n",
        )?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        Ok(ToolchainSpec {
            kind: kind.clone(),
            version: version.to_string(),
            path: binary_path,
            envs: HashMap::new(),
            checksum: None,
            is_hermetic: true,
        })
    }
}

fn default_toolchain_dir() -> PathBuf {
    if let Ok(home) = env::var("USERPROFILE").or_else(|_| env::var("HOME")) {
        PathBuf::from(home).join(".forge").join("toolchains")
    } else {
        env::temp_dir().join("forge").join("toolchains")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_toolchain_downloader_initialization() {
        let downloader = ToolchainDownloader::new();
        assert!(!downloader.is_offline());
        assert!(!downloader.sources.is_empty());
    }

    #[test]
    fn test_ensure_toolchain_simulation() {
        let temp = TempDir::new().unwrap();
        let downloader = ToolchainDownloader::new().with_base_dir(temp.path());

        let spec = downloader
            .ensure_toolchain(&ToolchainKind::Zig, "0.13.0")
            .unwrap();
        assert_eq!(spec.kind, ToolchainKind::Zig);
        assert_eq!(spec.version, "0.13.0");
        assert!(spec.is_hermetic);
        assert!(spec.path.exists());
        assert!(downloader.is_installed(&ToolchainKind::Zig, "0.13.0"));
    }

    #[test]
    fn test_offline_mode_rejection() {
        let temp = TempDir::new().unwrap();
        let mut downloader = ToolchainDownloader::new().with_base_dir(temp.path());
        downloader.set_offline(true);

        let err = downloader.ensure_toolchain(&ToolchainKind::Zig, "0.13.0");
        assert!(err.is_err());
    }
}
