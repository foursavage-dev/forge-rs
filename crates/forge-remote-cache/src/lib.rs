use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use thiserror::Error;

use forge_cache::{CacheError, LocalCache};

#[derive(Debug, Error)]
pub enum RemoteCacheError {
    #[error("network error: {0}")]
    Network(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("local cache error: {0}")]
    Local(#[from] CacheError),
}

pub trait RemoteCacheClient: Debug + Send + Sync {
    fn get_fingerprint(&self, key: &str) -> Result<Option<String>, RemoteCacheError>;
    fn put_fingerprint(&self, key: &str, fingerprint: &str) -> Result<(), RemoteCacheError>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryRemoteCache {
    store: Arc<Mutex<HashMap<String, String>>>,
}

impl InMemoryRemoteCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl RemoteCacheClient for InMemoryRemoteCache {
    fn get_fingerprint(&self, key: &str) -> Result<Option<String>, RemoteCacheError> {
        let guard = self
            .store
            .lock()
            .map_err(|e| RemoteCacheError::Network(e.to_string()))?;
        Ok(guard.get(key).cloned())
    }

    fn put_fingerprint(&self, key: &str, fingerprint: &str) -> Result<(), RemoteCacheError> {
        let mut guard = self
            .store
            .lock()
            .map_err(|e| RemoteCacheError::Network(e.to_string()))?;
        guard.insert(key.to_string(), fingerprint.to_string());
        Ok(())
    }
}

#[derive(Debug)]
pub struct CompositeCache {
    pub local: LocalCache,
    pub remote: Option<Box<dyn RemoteCacheClient>>,
}

impl CompositeCache {
    pub fn new(local: LocalCache, remote: Option<Box<dyn RemoteCacheClient>>) -> Self {
        Self { local, remote }
    }

    pub fn matches(&self, key: &str, expected_fingerprint: &str) -> bool {
        if self.local.matches(key, expected_fingerprint) {
            return true;
        }

        if let Some(remote) = &self.remote {
            if let Ok(Some(remote_fp)) = remote.get_fingerprint(key) {
                if remote_fp == expected_fingerprint {
                    let _ = self.local.put(key, expected_fingerprint);
                    return true;
                }
            }
        }

        false
    }

    pub fn put(&self, key: &str, fingerprint: &str) {
        let _ = self.local.put(key, fingerprint);
        if let Some(remote) = &self.remote {
            let _ = remote.put_fingerprint(key, fingerprint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_tiered_composite_cache_fallback() {
        let temp = tempdir().unwrap();
        let local = LocalCache::new(temp.path()).unwrap();
        let remote = InMemoryRemoteCache::new();

        remote.put_fingerprint("task_1", "hash_abc").unwrap();

        let composite = CompositeCache::new(local, Some(Box::new(remote)));

        assert!(composite.matches("task_1", "hash_abc"));
        assert!(!composite.matches("task_1", "hash_wrong"));

        assert!(composite.local.matches("task_1", "hash_abc"));
    }
}
