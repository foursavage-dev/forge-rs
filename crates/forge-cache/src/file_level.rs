#![forbid(unsafe_code)]

//! File-level cache granularity
//!
//! This module provides fine-grained caching at the file level rather than
//! package level, allowing incremental builds when only specific files change.
//!
//! Performance optimizations:
//! - DashMap for lock-free concurrent access
//! - Cache-friendly memory layout
//! - Reduced allocation overhead

use std::path::{Path, PathBuf};
use std::sync::Arc;

use dashmap::DashMap;
use forge_cas::{Artifact, ArtifactHash};

#[derive(Debug, Clone)]
pub struct FileLevelCache {
    /// Map of file hashes to their artifacts (DashMap for lock-free concurrent access)
    file_artifacts: Arc<DashMap<String, ArtifactHash>>,
    /// CAS storage interface (DashMap for concurrent access)
    cas_storage: Arc<DashMap<ArtifactHash, Artifact>>,
}

impl Default for FileLevelCache {
    fn default() -> Self {
        Self::new()
    }
}

impl FileLevelCache {
    pub fn new() -> Self {
        Self {
            file_artifacts: Arc::new(DashMap::new()),
            cas_storage: Arc::new(DashMap::new()),
        }
    }

    /// Check if a file is cached at the file level (lock-free read)
    pub fn is_file_cached(&self, file_path: &Path) -> bool {
        let file_key = self.file_key(file_path);
        self.file_artifacts.contains_key(&file_key)
    }

    /// Get cached artifact for a specific file (lock-free read)
    pub fn get_file_artifact(&self, file_path: &Path) -> Option<Artifact> {
        let file_key = self.file_key(file_path);

        if let Some(hash) = self.file_artifacts.get(&file_key) {
            self.cas_storage
                .get(hash.value())
                .map(|artifact| artifact.clone())
        } else {
            None
        }
    }

    /// Cache a file artifact (lock-free write with DashMap)
    pub fn cache_file(
        &self,
        file_path: &Path,
        artifact: Artifact,
    ) -> Result<(), anyhow::Error> {
        let hash = artifact.metadata.hash.clone();
        let file_key = self.file_key(file_path);

        // Store in CAS (DashMap insert is lock-free)
        self.cas_storage.insert(hash.clone(), artifact);

        // Update file mapping (DashMap insert is lock-free)
        self.file_artifacts.insert(file_key, hash);

        Ok(())
    }

    /// Invalidate cache for a specific file (lock-free operation)
    pub fn invalidate_file(&self, file_path: &Path) {
        let file_key = self.file_key(file_path);
        self.file_artifacts.remove(&file_key);
    }

    /// Invalidate cache for all files in a directory (concurrent-safe iteration)
    pub fn invalidate_directory(&self, dir_path: &Path) {
        let dir_key = dir_path.to_string_lossy().to_string();

        // Use retain for efficient concurrent filtering
        self.file_artifacts
            .retain(|key, _| !key.starts_with(&dir_key));
    }

    /// Generate a cache key for a file
    fn file_key(&self, file_path: &Path) -> String {
        file_path.to_string_lossy().to_string()
    }

    /// Get statistics about file-level cache (lock-free read)
    pub fn stats(&self) -> FileCacheStats {
        FileCacheStats {
            total_files: self.file_artifacts.len(),
            total_artifacts: self.cas_storage.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileCacheStats {
    pub total_files: usize,
    pub total_artifacts: usize,
}

/// File-level dependency tracking with lock-free concurrent access
#[derive(Debug, Clone)]
pub struct FileDependencyGraph {
    /// Map of files to their dependencies (DashMap for lock-free concurrent access)
    dependencies: Arc<DashMap<PathBuf, Vec<PathBuf>>>,
    /// Reverse map for quick invalidation (DashMap for lock-free concurrent access)
    reverse_dependencies: Arc<DashMap<PathBuf, Vec<PathBuf>>>,
}

impl Default for FileDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl FileDependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: Arc::new(DashMap::new()),
            reverse_dependencies: Arc::new(DashMap::new()),
        }
    }

    /// Add a dependency relationship (lock-free concurrent write)
    pub fn add_dependency(&self, file: PathBuf, depends_on: PathBuf) {
        // DashMap entry API for concurrent-safe modification
        self.dependencies
            .entry(file.clone())
            .and_modify(|deps| deps.push(depends_on.clone()))
            .or_insert_with(|| vec![depends_on.clone()]);
        self.reverse_dependencies
            .entry(depends_on)
            .and_modify(|rev| rev.push(file.clone()))
            .or_insert_with(|| vec![file]);
    }

    /// Get files that depend on a given file (lock-free read)
    pub fn get_dependents(&self, file: &Path) -> Vec<PathBuf> {
        self.reverse_dependencies
            .get(file)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Invalidate file and all its dependents (concurrent-safe)
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
    pub fn parse_dep_file(&self, dep_file: &Path) -> Result<(), anyhow::Error> {
        let content = std::fs::read_to_string(dep_file)?;

        // Parse .d file format: target: dependencies
        if let Some(colon_pos) = content.find(':') {
            let target = content[..colon_pos].trim();
            let deps_str = content[colon_pos + 1..].trim();

            let target_path = PathBuf::from(target);
            let dependencies: Vec<PathBuf> =
                deps_str.split_whitespace().map(PathBuf::from).collect();

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
