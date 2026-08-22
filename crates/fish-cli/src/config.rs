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
    Java,
    Kotlin,
    Dotnet,
    #[serde(rename = "csharp")]
    CSharp,
    #[serde(rename = "fsharp")]
    FSharp,
    Swift,
    #[serde(rename = "objc")]
    ObjC,
    #[serde(rename = "objective-c")]
    ObjectiveC,
    Dart,
    Flutter,
    Zig,
    Docker,
    #[serde(rename = "oci")]
    Oci,
    Plugin,
    #[serde(rename = "rules")]
    Rules,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FishConfig {
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
    #[serde(default)]
    pub remote_cache: Option<String>,
    #[serde(default)]
    pub remote_cache_token: Option<String>,
    #[serde(default)]
    pub remote_workers: Option<Vec<String>>,
    #[serde(default)]
    pub remote_workers_token: Option<String>,
    #[serde(default)]
    pub cache_dir: Option<String>,
    #[serde(default)]
    pub send_source: bool,
    #[serde(default)]
    pub ram_limit: Option<u8>,
    #[serde(default)]
    pub semantic: bool,
    #[serde(default)]
    pub ramdisk: bool,
    #[serde(default)]
    pub swarm: bool,
    #[serde(default)]
    pub reflink: bool,
    #[serde(default)]
    pub hermetic_trace: bool,
    #[serde(default)]
    pub swarm_compute: bool,
    #[serde(default)]
    pub critical_path: bool,
    #[serde(default)]
    pub turbo_link: bool,
    #[serde(default)]
    pub speculative: bool,
    #[serde(default)]
    pub daemon_pool: bool,
    #[serde(default)]
    pub kernel_bypass: bool,
    #[serde(default)]
    pub wasm_sandbox: bool,
    #[serde(default)]
    pub super_opt: bool,
    #[serde(default)]
    pub otel_endpoint: Option<String>,
}

impl FishConfig {
    pub fn load(start_dir: &Path) -> Result<Option<FishConfig>, String> {
        let path = start_dir.join("fish.toml");
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
        assert!(FishConfig::load(dir.path()).unwrap().is_none());
    }

    #[test]
    fn empty_file_yields_defaults() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("fish.toml"), "").unwrap();
        let config = FishConfig::load(dir.path()).unwrap().unwrap();
        assert_eq!(config.backend, BackendChoice::Auto);
        assert_eq!(config.jobs, None);
        assert!(!config.no_cache);
    }

    #[test]
    fn parses_every_field() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("fish.toml"),
            "backend = \"cc\"\njobs = 4\nno_cache = true\nremote_cache = \"127.0.0.1:9091\"\nremote_workers = [\"127.0.0.1:9092\"]\n",
        )
        .unwrap();
        let config = FishConfig::load(dir.path()).unwrap().unwrap();
        assert_eq!(config.backend, BackendChoice::Cc);
        assert_eq!(config.jobs, Some(4));
        assert!(config.no_cache);
        assert_eq!(config.remote_cache.as_deref(), Some("127.0.0.1:9091"));
        assert_eq!(
            config.remote_workers.as_deref(),
            Some(&["127.0.0.1:9092".to_string()][..])
        );
    }

    #[test]
    fn unknown_keys_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("fish.toml"), "mode = \"release\"\n").unwrap();
        let error = FishConfig::load(dir.path()).unwrap_err();
        assert!(error.contains("invalid `"), "got: {error}");
    }

    #[test]
    fn zero_jobs_means_auto() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("fish.toml"), "jobs = 0\n").unwrap();
        let config = FishConfig::load(dir.path()).unwrap().unwrap();

        assert_eq!(config.jobs.filter(|&j| j > 0), None);
    }

    #[test]
    fn garbage_is_an_error_not_a_silent_default() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("fish.toml"), "not toml {{{").unwrap();
        assert!(FishConfig::load(dir.path()).is_err());
    }
}
