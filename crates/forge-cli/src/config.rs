//! Optional per-project `forge.toml` configuration.
//!
//! Forge looks for a `forge.toml` next to the project it is building (the
//! same directory where `forge.cc.json` / `forge.go.json` live). The file
//! only tunes knobs that already exist on the command line, so the CLI and
//! the file can never disagree about what is possible:
//!
//! ```toml
//! # Backend to use: "auto" (default) picks Cargo for Cargo.toml, the C/C++
//! # backend for forge.cc.json, Go for go.mod, and the TypeScript backend
//! # for forge.ts.json / package.json.
//! backend = "auto"      # auto | rust | cc | go | ts | typescript | js | javascript
//!
//! # Worker processes; 0 (default) means one per logical CPU.
//! jobs = 0
//!
//! # Disable the fingerprint cache for every run in this project.
//! no_cache = false
//!
//! # Sandbox every spawned tool with a clean environment.
//! sandbox = false
//!
//! # Kill tasks that run longer than this many seconds.
//! timeout = 60
//!
//! # Write a Chrome trace of the run to this file.
//! profile = "forge-trace.json"
//! ```
//!
//! Command-line flags always win over the file: passing `-j 2` beats
//! `jobs = 8`.

use std::path::Path;

use serde::Deserialize;

/// Backend selection from `forge.toml`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendChoice {
    #[default]
    Auto,
    Rust,
    Cc,
    Go,
    Ts,
    #[serde(rename = "typescript")]
    Typescript,
    #[serde(rename = "javascript")]
    Javascript,
    #[serde(rename = "js")]
    Js,
}

/// Parsed `forge.toml`. All fields are optional; absent fields keep their
/// command-line or built-in defaults.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForgeConfig {
    #[serde(default)]
    pub backend: BackendChoice,
    #[serde(default)]
    pub jobs: Option<usize>,
    #[serde(default)]
    pub no_cache: bool,
    #[serde(default)]
    pub sandbox: bool,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub profile: Option<String>,
}

impl ForgeConfig {
    /// Read `forge.toml` from `start_dir`.
    ///
    /// Returns `Ok(None)` when no file exists; a present-but-unparseable
    /// file is an error — silently ignoring a config that can no longer be
    /// read would hide typos and format drift.
    pub fn load(start_dir: &Path) -> Result<Option<ForgeConfig>, String> {
        let path = start_dir.join("forge.toml");
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|error| format!("cannot read `{}`: {error}", path.display()))?;
        toml::from_str(&content)
            .map(Some)
            .map_err(|error| format!("invalid `{}`: {error}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_absent() {
        let dir = tempfile::tempdir().unwrap();
        assert!(ForgeConfig::load(dir.path()).unwrap().is_none());
    }

    #[test]
    fn empty_file_yields_defaults() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("forge.toml"), "").unwrap();
        let config = ForgeConfig::load(dir.path()).unwrap().unwrap();
        assert_eq!(config.backend, BackendChoice::Auto);
        assert_eq!(config.jobs, None);
        assert!(!config.no_cache);
    }

    #[test]
    fn parses_every_field() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("forge.toml"),
            "backend = \"cc\"\njobs = 4\nno_cache = true\n",
        )
        .unwrap();
        let config = ForgeConfig::load(dir.path()).unwrap().unwrap();
        assert_eq!(config.backend, BackendChoice::Cc);
        assert_eq!(config.jobs, Some(4));
        assert!(config.no_cache);
    }

    #[test]
    fn unknown_keys_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("forge.toml"), "mode = \"release\"\n").unwrap();
        let error = ForgeConfig::load(dir.path()).unwrap_err();
        assert!(error.contains("invalid `"), "got: {error}");
    }

    #[test]
    fn zero_jobs_means_auto() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("forge.toml"), "jobs = 0\n").unwrap();
        let config = ForgeConfig::load(dir.path()).unwrap().unwrap();
        // The merge in the CLI turns 0 (auto) into no override at all.
        assert_eq!(config.jobs.filter(|&j| j > 0), None);
    }

    #[test]
    fn garbage_is_an_error_not_a_silent_default() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("forge.toml"), "not toml {{{").unwrap();
        assert!(ForgeConfig::load(dir.path()).is_err());
    }
}
