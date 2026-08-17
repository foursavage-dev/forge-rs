#![forbid(unsafe_code)]

use crate::config::DockerProjectConfig;
use forge_core::{DEFAULT_EXCLUDED_DIRS, FingerprintUtils};

#[derive(Debug, Clone)]
pub struct DockerFingerprinter {
    config: DockerProjectConfig,
}

impl DockerFingerprinter {
    pub fn new(config: DockerProjectConfig) -> Self {
        Self { config }
    }

    pub fn compute(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut hasher = blake3::Hasher::new();

        if let Some(dockerfile) = &self.config.dockerfile_path {
            if dockerfile.exists() {
                let _ = FingerprintUtils::hash_file_into(dockerfile, &mut hasher);
            }
        }

        if self.config.context_path.is_dir() {
            FingerprintUtils::hash_directory_filtered(
                &self.config.context_path,
                |dir_name| DEFAULT_EXCLUDED_DIRS.contains(&dir_name) || dir_name == ".dockerignore",
                |_file_path| true,
                &mut hasher,
            )?;
        }

        let mut args: Vec<_> = self.config.build_args.iter().collect();
        args.sort_by_key(|(k, _)| *k);
        for (key, value) in args {
            hasher.update(key.as_bytes());
            hasher.update(value.as_bytes());
        }

        if let Some(target) = &self.config.target {
            hasher.update(target.as_bytes());
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    pub fn compute_layer_fingerprint(&self, line: &str) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(line.as_bytes());
        hasher.finalize().to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprinter_creation() {
        let config = DockerProjectConfig {
            dockerfile_path: None,
            context_path: std::path::PathBuf::from("."),
            build_args: std::collections::HashMap::new(),
            target: None,
            cache_from: Vec::new(),
            cache_to: Vec::new(),
        };

        let fingerprinter = DockerFingerprinter::new(config);
        let fingerprint = fingerprinter.compute();
        assert!(fingerprint.is_ok());
    }

    #[test]
    fn test_layer_fingerprint() {
        let config = DockerProjectConfig {
            dockerfile_path: None,
            context_path: std::path::PathBuf::from("."),
            build_args: std::collections::HashMap::new(),
            target: None,
            cache_from: Vec::new(),
            cache_to: Vec::new(),
        };

        let fingerprinter = DockerFingerprinter::new(config);
        let fp1 = fingerprinter.compute_layer_fingerprint("RUN apt-get update");
        let fp2 = fingerprinter.compute_layer_fingerprint("RUN apt-get update");
        let fp3 = fingerprinter.compute_layer_fingerprint("RUN apt-get upgrade");

        assert_eq!(fp1, fp2);
        assert_ne!(fp1, fp3);
    }
}
