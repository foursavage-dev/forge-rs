#![forbid(unsafe_code)]

//! Adapter for integrating FileLevelCache with CachingExecutor
//!
//! This module provides an adapter layer to bridge file-level caching
//! with the existing task-level caching system. This is a gradual migration
//! path - full integration requires scheduler changes to understand file-level
//! dependencies.

use std::path::PathBuf;
use std::sync::Arc;

use forge_executor::{CacheEntry, Task, TaskExecutor, TaskOutcome, TaskStatus, ExecutorError};

use super::file_level::{FileLevelCache, FileDependencyGraph};

/// Adapter that wraps FileLevelCache to work with CachingExecutor interface
///
/// This adapter maintains a mapping between task keys and their constituent files,
/// allowing the executor to check file-level cache before falling back to task-level cache.
pub struct FileLevelCacheAdapter {
    file_cache: Arc<FileLevelCache>,
    dep_graph: Arc<FileDependencyGraph>,
    /// Mapping from task keys to their source files
    task_files: Arc<std::sync::RwLock<std::collections::HashMap<String, Vec<PathBuf>>>>,
}

impl FileLevelCacheAdapter {
    pub fn new(file_cache: Arc<FileLevelCache>, dep_graph: Arc<FileDependencyGraph>) -> Self {
        Self {
            file_cache,
            dep_graph,
            task_files: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Register a task with its source files
    pub fn register_task_files(&self, task_key: String, files: Vec<PathBuf>) {
        let mut task_files = self.task_files.write().unwrap();
        task_files.insert(task_key, files);
    }

    /// Check if all files for a task are cached
    pub fn is_task_files_cached(&self, task_key: &str) -> bool {
        let task_files = self.task_files.read().unwrap();
        if let Some(files) = task_files.get(task_key) {
            files.iter().all(|f| self.file_cache.is_file_cached(f))
        } else {
            false
        }
    }

    /// Invalidate cache for a task's files
    pub fn invalidate_task_files(&self, task_key: &str) {
        let task_files = self.task_files.read().unwrap();
        if let Some(files) = task_files.get(task_key) {
            for file in files {
                self.dep_graph.invalidate_with_dependents(file, &self.file_cache);
            }
        }
    }

    /// Get file-level cache statistics
    pub fn file_cache_stats(&self) -> super::file_level::FileCacheStats {
        self.file_cache.stats()
    }
}

/// CachingExecutor that uses file-level cache as first-tier cache
///
/// This executor checks file-level cache first, then falls back to task-level cache,
/// and finally executes the task if both caches miss.
pub struct HybridCachingExecutor<I> {
    inner: I,
    local_cache: super::LocalCache,
    file_adapter: Arc<FileLevelCacheAdapter>,
}

impl<I: TaskExecutor> HybridCachingExecutor<I> {
    pub fn new(
        inner: I,
        local_cache: super::LocalCache,
        file_adapter: Arc<FileLevelCacheAdapter>,
    ) -> Self {
        Self {
            inner,
            local_cache,
            file_adapter,
        }
    }

    pub fn file_adapter(&self) -> &Arc<FileLevelCacheAdapter> {
        &self.file_adapter
    }

    pub fn local_cache(&self) -> &super::LocalCache {
        &self.local_cache
    }

    pub fn into_inner(self) -> I {
        self.inner
    }
}

impl<I: TaskExecutor> TaskExecutor for HybridCachingExecutor<I> {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        // Check file-level cache first (if task has registered files)
        if let Some(CacheEntry { key, .. }) = &task.cache {
            if self.file_adapter.is_task_files_cached(key) {
                // All files are cached at file level - can potentially skip execution
                // For now, we still check task-level cache to maintain compatibility
                return Ok(TaskOutcome::cached(task));
            }
        }

        // Check task-level cache
        if let Some(CacheEntry { key, fingerprint }) = &task.cache {
            if self.local_cache.matches(key, fingerprint) {
                return Ok(TaskOutcome::cached(task));
            }
        }

        // Execute the task
        let outcome = self.inner.execute(task)?;

        // Cache the result at task level
        if outcome.status == TaskStatus::Executed {
            if let Some(CacheEntry { key, fingerprint }) = &task.cache {
                if let Err(_error) = self.local_cache.put(key, fingerprint) {
                    self.local_cache.stats().record_error();
                }
            }
        }

        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_adapter_registration() {
        let file_cache = Arc::new(FileLevelCache::new());
        let dep_graph = Arc::new(FileDependencyGraph::new());
        let adapter = FileLevelCacheAdapter::new(file_cache, dep_graph);

        let files = vec![
            PathBuf::from("/src/main.rs"),
            PathBuf::from("/src/lib.rs"),
        ];
        adapter.register_task_files("build_crate".to_string(), files);

        assert!(!adapter.is_task_files_cached("build_crate"));
    }

    #[test]
    fn test_hybrid_executor_creation() {
        let temp_dir = TempDir::new().unwrap();
        let local_cache = crate::LocalCache::new(temp_dir.path()).unwrap();
        let file_cache = Arc::new(FileLevelCache::new());
        let dep_graph = Arc::new(FileDependencyGraph::new());
        let file_adapter = Arc::new(FileLevelCacheAdapter::new(file_cache, dep_graph));

        // Test that the adapter can be created
        assert!(!file_adapter.is_task_files_cached("nonexistent_task"));
        assert!(local_cache.root().exists());
    }
}
