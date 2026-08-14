#![forbid(unsafe_code)]

use crate::config::DockerProjectConfig;
use blake3;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

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
        
        // Hash Dockerfile content
        if let Some(dockerfile) = &self.config.dockerfile_path {
            if dockerfile.exists() {
                let content = fs::read(dockerfile)?;
                hasher.update(&content);
            }
        }
        
        // Hash build context files
        let context_files = self.collect_context_files()?;
        for file in context_files {
            if let Ok(content) = fs::read(&file) {
                hasher.update(&content);
            }
        }
        
        // Hash build args
        let mut args: Vec<_> = self.config.build_args.iter().collect();
        args.sort_by_key(|(k, _)| *k);
        for (key, value) in args {
            hasher.update(key.as_bytes());
            hasher.update(value.as_bytes());
        }
        
        // Hash target stage
        if let Some(target) = &self.config.target {
            hasher.update(target.as_bytes());
        }
        
        Ok(hasher.finalize().to_hex().to_string())
    }
    
    fn collect_context_files(&self) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        let mut visited = HashSet::new();
        
        if !self.config.context_path.exists() {
            return Ok(files);
        }
        
        self.walk_directory(&self.config.context_path, &mut files, &mut visited)?;
        
        // Sort for consistent fingerprinting
        files.sort();
        
        Ok(files)
    }
    
    fn walk_directory(
        &self,
        dir: &Path,
        files: &mut Vec<std::path::PathBuf>,
        visited: &mut HashSet<std::path::PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let canonical = dir.canonicalize()?;
        
        if visited.contains(&canonical) {
            return Ok(());
        }
        visited.insert(canonical);
        
        let entries = fs::read_dir(dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            // Skip .dockerignore and other special files
            if let Some(name) = path.file_name() {
                if name == ".dockerignore" || name == ".git" {
                    continue;
                }
            }
            
            if path.is_dir() {
                self.walk_directory(&path, files, visited)?;
            } else if path.is_file() {
                files.push(path);
            }
        }
        
        Ok(())
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
