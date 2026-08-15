#![forbid(unsafe_code)]

//! File-level cache granularity
//! 
//! This module provides fine-grained caching at the file level rather than
//! package level, allowing incremental builds when only specific files change.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;

use forge_cas::{Artifact, ArtifactHash};

#[derive(Debug, Clone)]
pub struct FileLevelCache {
    /// Map of file hashes to their artifacts
    file_artifacts: Arc<RwLock<HashMap<String, ArtifactHash>>>,
    /// CAS storage interface (simplified)
    cas_storage: Arc<RwLock<HashMap<ArtifactHash, Artifact>>>,
}

impl Default for FileLevelCache {
    fn default() -> Self {
        Self::new()
    }
}

impl FileLevelCache {
    pub fn new() -> Self {
        Self {
            file_artifacts: Arc::new(RwLock::new(HashMap::new())),
            cas_storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a file is cached at the file level
    pub fn is_file_cached(&self, file_path: &Path) -> bool {
        let artifacts = self.file_artifacts.read().unwrap();
        let file_key = self.file_key(file_path);
        artifacts.contains_key(&file_key)
    }

    /// Get cached artifact for a specific file
    pub fn get_file_artifact(&self, file_path: &Path) -> Option<Artifact> {
        let artifacts = self.file_artifacts.read().unwrap();
        let file_key = self.file_key(file_path);
        
        if let Some(hash) = artifacts.get(&file_key) {
            let storage = self.cas_storage.read().unwrap();
            storage.get(hash).cloned()
        } else {
            None
        }
    }

    /// Cache a file artifact
    pub fn cache_file(&self, file_path: &Path, artifact: Artifact) -> Result<(), Box<dyn std::error::Error>> {
        let hash = artifact.metadata.hash.clone();
        let file_key = self.file_key(file_path);
        
        // Store in CAS
        let mut storage = self.cas_storage.write().unwrap();
        storage.insert(hash.clone(), artifact);
        
        // Update file mapping
        let mut artifacts = self.file_artifacts.write().unwrap();
        artifacts.insert(file_key, hash);
        
        Ok(())
    }

    /// Invalidate cache for a specific file
    pub fn invalidate_file(&self, file_path: &Path) {
        let file_key = self.file_key(file_path);
        let mut artifacts = self.file_artifacts.write().unwrap();
        artifacts.remove(&file_key);
    }

    /// Invalidate cache for all files in a directory
    pub fn invalidate_directory(&self, dir_path: &Path) {
        let dir_key = dir_path.to_string_lossy().to_string();
        let mut artifacts = self.file_artifacts.write().unwrap();
        
        artifacts.retain(|key, _| !key.starts_with(&dir_key));
    }

    /// Generate a cache key for a file
    fn file_key(&self, file_path: &Path) -> String {
        file_path.to_string_lossy().to_string()
    }

    /// Get statistics about file-level cache
    pub fn stats(&self) -> FileCacheStats {
        let artifacts = self.file_artifacts.read().unwrap();
        FileCacheStats {
            total_files: artifacts.len(),
            total_artifacts: artifacts.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileCacheStats {
    pub total_files: usize,
    pub total_artifacts: usize,
}

/// File-level dependency tracking
#[derive(Debug, Clone)]
pub struct FileDependencyGraph {
    /// Map of files to their dependencies
    dependencies: Arc<RwLock<HashMap<PathBuf, Vec<PathBuf>>>>,
    /// Reverse map for quick invalidation
    reverse_dependencies: Arc<RwLock<HashMap<PathBuf, Vec<PathBuf>>>>,
}

impl Default for FileDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl FileDependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            reverse_dependencies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a dependency relationship
    pub fn add_dependency(&self, file: PathBuf, depends_on: PathBuf) {
        let mut deps = self.dependencies.write().unwrap();
        let mut reverse = self.reverse_dependencies.write().unwrap();
        
        deps.entry(file.clone()).or_default().push(depends_on.clone());
        reverse.entry(depends_on).or_default().push(file);
    }

    /// Get files that depend on a given file
    pub fn get_dependents(&self, file: &Path) -> Vec<PathBuf> {
        let reverse = self.reverse_dependencies.read().unwrap();
        reverse.get(file).cloned().unwrap_or_default()
    }

    /// Invalidate file and all its dependents
    pub fn invalidate_with_dependents(&self, file: &Path, cache: &FileLevelCache) {
        let dependents = self.get_dependents(file);
        
        // Invalidate the file itself
        cache.invalidate_file(file);
        
        // Invalidate all dependents
        for dependent in dependents {
            cache.invalidate_file(&dependent);
        }
    }

    /// Parse dependency file (e.g., .d files from GCC)
    pub fn parse_dep_file(&self, dep_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(dep_file)?;
        
        // Parse .d file format: target: dependencies
        if let Some(colon_pos) = content.find(':') {
            let target = content[..colon_pos].trim();
            let deps_str = content[colon_pos + 1..].trim();
            
            let target_path = PathBuf::from(target);
            let dependencies: Vec<PathBuf> = deps_str
                .split_whitespace()
                .map(PathBuf::from)
                .collect();
            
            for dep in dependencies {
                self.add_dependency(target_path.clone(), dep);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_file_cache_operations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();
        
        let cache = FileLevelCache::new();
        
        assert!(!cache.is_file_cached(&test_file));
    }

    #[test]
    fn test_dependency_graph() {
        let graph = FileDependencyGraph::new();
        
        let file1 = PathBuf::from("/a/file1.rs");
        let file2 = PathBuf::from("/a/file2.rs");
        
        graph.add_dependency(file1.clone(), file2.clone());
        
        let dependents = graph.get_dependents(&file2);
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], file1);
    }
}