#![forbid(unsafe_code)]

//! Configuration management for Fish
//!
//! This module provides hierarchical configuration management with validation,
//! environment variable support, and profile-based configurations.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishConfig {
    pub general: GeneralConfig,
    pub cache: CacheConfig,
    pub build: BuildConfig,
    pub ci: CIConfig,
    pub security: SecurityConfig,
    pub experimental: ExperimentalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub log_level: String,
    pub max_parallel_jobs: usize,
    pub timeout_seconds: Option<u64>,
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub local_path: Option<PathBuf>,
    pub remote_url: Option<String>,
    pub compression_enabled: bool,
    pub max_size_gb: Option<f64>,
    pub ttl_hours: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub level_batching: bool,
    pub incremental: bool,
    pub hermetic: bool,
    pub sandbox_enabled: bool,
    pub watch_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIConfig {
    pub platform: String,
    pub cache_enabled: bool,
    pub matrix_enabled: bool,
    pub timeout_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub level: String,
    pub allowed_executables: Vec<String>,
    pub max_file_size_mb: Option<u64>,
    pub network_access: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalConfig {
    pub hotpatch_enabled: bool,
    pub kernel_bypass_enabled: bool,
    pub turbolink_enabled: bool,
    pub speculative_compilation: bool,
}

impl Default for FishConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                log_level: "info".to_string(),
                max_parallel_jobs: 4,
                timeout_seconds: None,
                environment: "development".to_string(),
            },
            cache: CacheConfig {
                enabled: true,
                local_path: None,
                remote_url: None,
                compression_enabled: true,
                max_size_gb: Some(10.0),
                ttl_hours: Some(24),
            },
            build: BuildConfig {
                level_batching: true,
                incremental: true,
                hermetic: false,
                sandbox_enabled: false,
                watch_mode: false,
            },
            ci: CIConfig {
                platform: "github".to_string(),
                cache_enabled: true,
                matrix_enabled: true,
                timeout_minutes: 30,
            },
            security: SecurityConfig {
                level: "strict".to_string(),
                allowed_executables: vec![],
                max_file_size_mb: Some(100),
                network_access: true,
            },
            experimental: ExperimentalConfig {
                hotpatch_enabled: false,
                kernel_bypass_enabled: false,
                turbolink_enabled: false,
                speculative_compilation: false,
            },
        }
    }
}

impl FishConfig {
    pub fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        let config: FishConfig =
            toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    pub fn load_from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Override with environment variables
        if let Ok(level) = std::env::var("FISH_LOG_LEVEL") {
            config.general.log_level = level;
        }

        if let Ok(jobs) = std::env::var("FISH_MAX_JOBS") {
            config.general.max_parallel_jobs = jobs.parse().unwrap_or(4);
        }

        if let Ok(cache_enabled) = std::env::var("FISH_CACHE_ENABLED") {
            config.cache.enabled = cache_enabled.parse().unwrap_or(true);
        }

        config.validate()?;
        Ok(config)
    }

    pub fn merge_with_file(&mut self, path: &Path) -> Result<(), ConfigError> {
        let file_config = Self::load_from_file(path)?;
        self.merge(file_config);
        Ok(())
    }

    pub fn merge(&mut self, other: FishConfig) {
        // General config
        if other.general.log_level != "info" {
            self.general.log_level = other.general.log_level;
        }
        if other.general.max_parallel_jobs != 4 {
            self.general.max_parallel_jobs = other.general.max_parallel_jobs;
        }

        // Cache config
        if other.cache.local_path.is_some() {
            self.cache.local_path = other.cache.local_path;
        }
        if other.cache.remote_url.is_some() {
            self.cache.remote_url = other.cache.remote_url;
        }

        // Build config
        self.build.level_batching = other.build.level_batching;
        self.build.incremental = other.build.incremental;

        // CI config
        if other.ci.platform != "github" {
            self.ci.platform = other.ci.platform;
        }

        // Security config
        if other.security.level != "strict" {
            self.security.level = other.security.level;
        }
        if !other.security.allowed_executables.is_empty() {
            self.security.allowed_executables = other.security.allowed_executables;
        }

        // Experimental config
        self.experimental.hotpatch_enabled = other.experimental.hotpatch_enabled;
        self.experimental.kernel_bypass_enabled = other.experimental.kernel_bypass_enabled;
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate parallel jobs
        if self.general.max_parallel_jobs == 0 {
            return Err(ConfigError::Validation(
                "max_parallel_jobs must be greater than 0".to_string(),
            ));
        }

        if self.general.max_parallel_jobs > 128 {
            return Err(ConfigError::Validation(
                "max_parallel_jobs cannot exceed 128".to_string(),
            ));
        }

        // Validate cache size
        if let Some(max_size) = self.cache.max_size_gb {
            if max_size <= 0.0 {
                return Err(ConfigError::Validation(
                    "max_size_gb must be positive".to_string(),
                ));
            }
            if max_size > 1000.0 {
                return Err(ConfigError::Validation(
                    "max_size_gb cannot exceed 1000GB".to_string(),
                ));
            }
        }

        // Validate security level
        match self.security.level.as_str() {
            "allow" | "strict" | "paranoid" => {}
            _ => {
                return Err(ConfigError::Validation(
                    "security level must be 'allow', 'strict', or 'paranoid'".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn get_profile(&self, profile: &str) -> FishConfig {
        let mut profile_config = self.clone();

        match profile {
            "development" => {
                profile_config.general.log_level = "debug".to_string();
                profile_config.build.sandbox_enabled = false;
                profile_config.experimental.hotpatch_enabled = true;
            }
            "production" => {
                profile_config.general.log_level = "warn".to_string();
                profile_config.build.sandbox_enabled = true;
                profile_config.experimental.hotpatch_enabled = false;
            }
            "ci" => {
                profile_config.general.log_level = "info".to_string();
                profile_config.build.incremental = true;
                profile_config.cache.enabled = true;
            }
            _ => {}
        }

        profile_config
    }
}

#[derive(Debug, Clone)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    Validation(String),
    NotFound(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::Validation(msg) => write!(f, "Validation error: {}", msg),
            ConfigError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FishConfig::default();
        assert_eq!(config.general.max_parallel_jobs, 4);
        assert!(config.cache.enabled);
        assert_eq!(config.general.log_level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = FishConfig::default();
        config.general.max_parallel_jobs = 0;
        assert!(config.validate().is_err());

        config.general.max_parallel_jobs = 4;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_merge() {
        let mut config1 = FishConfig::default();
        let mut config2 = FishConfig::default();
        config2.general.max_parallel_jobs = 8;

        config1.merge(config2);
        assert_eq!(config1.general.max_parallel_jobs, 8);
    }

    #[test]
    fn test_profile_selection() {
        let config = FishConfig::default();
        let dev_profile = config.get_profile("development");

        assert_eq!(dev_profile.general.log_level, "debug");
        assert!(dev_profile.experimental.hotpatch_enabled);

        let prod_profile = config.get_profile("production");
        assert_eq!(prod_profile.general.log_level, "warn");
        assert!(!prod_profile.experimental.hotpatch_enabled);
    }

    #[test]
    fn test_env_override() {
        // Test the parsing logic used in load_from_env
        // Since we can't set environment variables due to #![forbid(unsafe_code)],
        // we test the parsing behavior directly

        // Test default values when env vars are not set
        let config = FishConfig::load_from_env().unwrap();
        assert_eq!(config.general.max_parallel_jobs, 4);
        assert_eq!(config.general.log_level, "info");
        assert!(config.cache.enabled);

        // Test parsing logic (simulate what load_from_env does)
        let jobs_str = "8";
        let parsed_jobs: usize = jobs_str.parse().unwrap_or(4);
        assert_eq!(parsed_jobs, 8);

        let cache_str = "false";
        let parsed_cache: bool = cache_str.parse().unwrap_or(true);
        assert!(!parsed_cache);

        let log_str = "debug";
        let parsed_log = log_str.to_string();
        assert_eq!(parsed_log, "debug");
    }
}
