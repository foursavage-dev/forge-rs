use std::path::Path;

use serde::Deserialize;

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
    Py,
    #[serde(rename = "python")]
    Python,
    Plugin,
    #[serde(rename = "rules")]
    Rules,
}

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
    #[serde(default)]
    pub tui: bool,
}

impl ForgeConfig {
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

        assert_eq!(config.jobs.filter(|&j| j > 0), None);
    }

    #[test]
    fn garbage_is_an_error_not_a_silent_default() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("forge.toml"), "not toml {{{").unwrap();
        assert!(ForgeConfig::load(dir.path()).is_err());
    }
}
