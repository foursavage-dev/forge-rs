#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerProjectConfig {
    /// Path to Dockerfile
    pub dockerfile_path: Option<PathBuf>,
    /// Build context path
    pub context_path: PathBuf,
    /// Build arguments
    pub build_args: HashMap<String, String>,
    /// Target stage for multi-stage builds
    pub target: Option<String>,
    /// Cache sources
    pub cache_from: Vec<String>,
    /// Cache destinations
    pub cache_to: Vec<String>,
}

impl DockerProjectConfig {
    pub fn from_dockerfile(dockerfile: PathBuf) -> Self {
        let context_path = dockerfile.parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();
        
        Self {
            dockerfile_path: Some(dockerfile),
            context_path,
            build_args: HashMap::new(),
            target: None,
            cache_from: Vec::new(),
            cache_to: Vec::new(),
        }
    }
    
    pub fn with_build_arg(mut self, key: String, value: String) -> Self {
        self.build_args.insert(key, value);
        self
    }
    
    pub fn with_target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }
    
    pub fn with_cache_from(mut self, cache: String) -> Self {
        self.cache_from.push(cache);
        self
    }
    
    pub fn with_cache_to(mut self, cache: String) -> Self {
        self.cache_to.push(cache);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_docker_config_creation() {
        let config = DockerProjectConfig::from_dockerfile(PathBuf::from("Dockerfile"));
        assert!(config.dockerfile_path.is_some());
        assert_eq!(config.build_args.len(), 0);
    }
    
    #[test]
    fn test_docker_config_build_args() {
        let config = DockerProjectConfig::from_dockerfile(PathBuf::from("Dockerfile"))
            .with_build_arg("VERSION".to_string(), "1.0".to_string());
        
        assert_eq!(config.build_args.get("VERSION"), Some(&"1.0".to_string()));
    }
}
