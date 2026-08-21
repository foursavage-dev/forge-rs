use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::toolchain::{ToolchainKind, ToolchainSpec};

/// A single path segment is safe when it cannot escape the directory it is
/// joined into. This guards `version`, `kind`, and `binary_rel_path` — all of
/// which are used to build filesystem paths below the toolchain base dir.
fn is_safe_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && !segment
            .bytes()
            .any(|byte| matches!(byte, b'/' | b'\\' | b'\0' | b':'))
}

/// A relative path made only of safe segments, with no absolute prefix.
fn is_safe_relative_path(path: &str) -> bool {
    !path.is_empty() && !path.starts_with('/') && path.split(['/', '\\']).all(is_safe_segment)
}

/// Directory name for a toolchain kind. `Custom` names are attacker-controlled
/// (they come from configuration), so they are validated and folded into a
/// fixed prefix instead of being interpolated verbatim.
fn toolchain_kind_name(kind: &ToolchainKind) -> String {
    match kind {
        ToolchainKind::Custom(name) if is_safe_segment(name) => format!("custom-{name}"),
        ToolchainKind::Custom(_) => "custom".to_string(),
        other => format!("{other:?}").to_lowercase(),
    }
}

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
        let base_dir = env::var("FISH_TOOLCHAIN_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_toolchain_dir());
        let offline = env::var("FISH_OFFLINE")
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
        // Refuse sources whose binary path or version could escape the
        // toolchain base directory when joined into a filesystem path.
        if !is_safe_segment(&source.version) || !is_safe_relative_path(&source.binary_rel_path) {
            return;
        }
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
        if !is_safe_segment(version) {
            return None;
        }
        let kind_str = toolchain_kind_name(kind);
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

        if !is_safe_segment(version) {
            return Err(anyhow!("invalid toolchain version `{version}`"));
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

        // Automatic download is not implemented. Failing loudly here prevents
        // the previous behaviour of installing a no-op stub (`exit 0`) that
        // silently turned every build into an empty success. The declared
        // `url` and `sha256` fields are reserved for the future downloader.
        let install_dir = self.base_dir.join(toolchain_kind_name(kind)).join(version);
        Err(anyhow!(
            "automatic download of toolchain {:?} {} (from {}) is not implemented; \
             install it manually under {:?}",
            kind,
            version,
            source.url,
            install_dir
        ))
    }
}

fn default_toolchain_dir() -> PathBuf {
    if let Ok(home) = env::var("USERPROFILE").or_else(|_| env::var("HOME")) {
        PathBuf::from(home).join(".fish").join("toolchains")
    } else {
        env::temp_dir().join("fish").join("toolchains")
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
    fn ensure_toolchain_fails_loudly_instead_of_installing_a_stub() {
        let temp = TempDir::new().unwrap();
        let downloader = ToolchainDownloader::new().with_base_dir(temp.path());

        let err = downloader.ensure_toolchain(&ToolchainKind::Zig, "0.13.0");
        assert!(
            err.is_err(),
            "uninstalled toolchains must error, not fake success"
        );
        assert!(
            !downloader.is_installed(&ToolchainKind::Zig, "0.13.0"),
            "no stub may be left behind"
        );
    }

    #[test]
    fn rejects_traversal_in_version_and_binary_rel_path() {
        let mut downloader = ToolchainDownloader::new();

        // A source whose binary path escapes the install dir is dropped.
        downloader.register_source(RemoteToolchainSource {
            kind: ToolchainKind::Python,
            version: "1.0".to_string(),
            url: "https://example.com/py.tgz".to_string(),
            sha256: None,
            binary_rel_path: "../../etc/passwd".to_string(),
        });
        assert!(
            !downloader
                .sources
                .contains_key(&(ToolchainKind::Python, "1.0".to_string())),
            "a source with a traversal binary path must not be registered"
        );

        // A traversal version is rejected at lookup time.
        assert!(
            downloader
                .get_installed_path(&ToolchainKind::Zig, "../../outside")
                .is_none()
        );
        assert!(
            downloader
                .ensure_toolchain(&ToolchainKind::Zig, "../../outside")
                .is_err()
        );
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
