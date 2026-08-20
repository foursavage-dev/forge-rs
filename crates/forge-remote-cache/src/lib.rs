#![forbid(unsafe_code)]

pub mod artifact;
pub mod client;
pub mod protocol;
pub mod reapi;
pub mod server;

use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thiserror::Error;

use forge_cache::{CacheError, LocalCache};
use forge_executor::{CacheEntry, ExecutorError, Task, TaskExecutor, TaskOutcome, TaskStatus};

pub use client::TcpRemoteCacheClient;
pub use protocol::{CacheRequest, CacheResponse};
pub use reapi::{
    ReapiAction, ReapiActionResult, ReapiClient, ReapiCommand, ReapiDigest, ReapiOutputFile,
};
pub use server::RemoteCacheServer;

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
    fn get_artifact(&self, key: &str) -> Result<Option<Vec<u8>>, RemoteCacheError> {
        let _ = key;
        Ok(None)
    }
    fn put_artifact(&self, key: &str, data: &[u8]) -> Result<(), RemoteCacheError> {
        let _ = (key, data);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryRemoteCache {
    store: Arc<Mutex<HashMap<String, String>>>,
    artifacts: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl InMemoryRemoteCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            artifacts: Arc::new(Mutex::new(HashMap::new())),
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

    fn get_artifact(&self, key: &str) -> Result<Option<Vec<u8>>, RemoteCacheError> {
        let guard = self
            .artifacts
            .lock()
            .map_err(|e| RemoteCacheError::Network(e.to_string()))?;
        Ok(guard.get(key).cloned())
    }

    fn put_artifact(&self, key: &str, data: &[u8]) -> Result<(), RemoteCacheError> {
        let mut guard = self
            .artifacts
            .lock()
            .map_err(|e| RemoteCacheError::Network(e.to_string()))?;
        guard.insert(key.to_string(), data.to_vec());
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

        if let Some(remote) = &self.remote
            && let Ok(Some(remote_fp)) = remote.get_fingerprint(key)
            && remote_fp == expected_fingerprint
        {
            let _ = self.local.put(key, expected_fingerprint);
            self.local.stats().record_hit();
            return true;
        }

        false
    }

    pub fn put(&self, key: &str, fingerprint: &str) {
        self.put_with_artifact(key, fingerprint, None);
    }

    pub fn put_with_artifact(&self, key: &str, fingerprint: &str, artifact_hash: Option<String>) {
        let _ = self
            .local
            .put_with_artifact(key, fingerprint, artifact_hash.clone());
        if let Some(remote) = &self.remote {
            let _ = remote.put_fingerprint(key, fingerprint);
            if let Some(hash) = artifact_hash
                && let Some(blob) = self.local.get_object(&hash)
            {
                let _ = remote.put_artifact(key, &blob);
            }
        }
    }

    /// Fetches the artifact blob for `key`: the local CAS first (via the
    /// recorded content hash), then the remote cache.
    pub fn get_artifact(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(hash) = self.local.artifact_hash(key)
            && let Some(blob) = self.local.get_object(&hash)
        {
            return Some(blob);
        }
        if let Some(remote) = &self.remote
            && let Ok(Some(blob)) = remote.get_artifact(key)
        {
            let _ = self.local.put_object(&blob_hash_of(&blob), &blob);
            return Some(blob);
        }
        None
    }
}

fn blob_hash_of(blob: &[u8]) -> String {
    artifact::blob_hash(blob)
}

#[derive(Debug)]
pub struct CompositeCachingExecutor<I> {
    inner: I,
    cache: CompositeCache,
}

impl<I: TaskExecutor> CompositeCachingExecutor<I> {
    pub fn new(inner: I, cache: CompositeCache) -> Self {
        Self { inner, cache }
    }

    pub fn cache(&self) -> &CompositeCache {
        &self.cache
    }

    pub fn into_inner(self) -> I {
        self.inner
    }

    fn restore_artifacts(&self, task: &Task) {
        if task.artifacts.is_empty() {
            return;
        }
        let Some(cache_key) = task.cache.as_ref().map(|e| e.key.clone()) else {
            return;
        };
        let Some(blob) = self.cache.get_artifact(&cache_key) else {
            return;
        };
        let root = task.spec.cwd.clone().unwrap_or_else(|| PathBuf::from("."));
        if let Err(error) = artifact::unpack_artifacts(&blob, &root) {
            self.cache.local.stats().record_error();
            let _ = error;
        }
    }

    fn store_artifacts(&self, task: &Task) -> Option<String> {
        if task.artifacts.is_empty() {
            return None;
        }
        let cache_key = task.cache.as_ref().map(|e| e.key.clone())?;
        let root = task.spec.cwd.clone().unwrap_or_else(|| PathBuf::from("."));
        let blob = match artifact::pack_artifacts(&root, &task.artifacts) {
            Ok(blob) => blob,
            Err(_) => return None,
        };
        let hash = artifact::blob_hash(&blob);
        let _ = self.cache.local.put_object(&hash, &blob);
        if let Some(remote) = &self.cache.remote {
            let _ = remote.put_artifact(&cache_key, &blob);
        }
        Some(hash)
    }
}

impl<I: TaskExecutor> TaskExecutor for CompositeCachingExecutor<I> {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        if let Some(CacheEntry { key, fingerprint }) = &task.cache
            && self.cache.matches(key, fingerprint)
        {
            self.restore_artifacts(task);
            return Ok(TaskOutcome::cached(task));
        }

        let outcome = self.inner.execute(task)?;
        if outcome.status == TaskStatus::Executed
            && let Some(CacheEntry { key, fingerprint }) = &task.cache
        {
            let artifact_hash = self.store_artifacts(task);
            self.cache
                .put_with_artifact(key, fingerprint, artifact_hash);
        }
        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;
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

    fn start_test_server(
        token: Option<String>,
        dir: Option<std::path::PathBuf>,
    ) -> (RemoteCacheServer, String, std::thread::JoinHandle<()>) {
        for _ in 0..10 {
            let addr = match TcpListener::bind("127.0.0.1:0") {
                Ok(l) => {
                    let a = l.local_addr().unwrap().to_string();
                    drop(l);
                    thread::sleep(Duration::from_millis(15));
                    a
                }
                Err(_) => continue,
            };
            let server = RemoteCacheServer::new(&addr, token.clone(), dir.clone());
            if let Ok(handle) = server.start_background() {
                thread::sleep(Duration::from_millis(50));
                let client = TcpRemoteCacheClient::new(&addr, token.clone());
                if token.is_some() || client.ping().is_ok() {
                    return (server, addr, handle);
                }
                server.stop();
                let _ = handle.join();
            }
        }
        panic!("Failed to start remote cache test server");
    }

    #[test]
    fn test_tcp_remote_cache_server_and_client() {
        let temp = tempdir().unwrap();
        let (server, addr, _server_handle) = start_test_server(
            Some("secret123".to_string()),
            Some(temp.path().to_path_buf()),
        );

        let client = TcpRemoteCacheClient::new(&addr, Some("secret123".to_string()));
        assert!(client.ping().unwrap());

        client.put_fingerprint("task_alpha", "fp_999").unwrap();
        let fetched = client.get_fingerprint("task_alpha").unwrap();
        assert_eq!(fetched.as_deref(), Some("fp_999"));

        let non_existent = client.get_fingerprint("non_existent").unwrap();
        assert_eq!(non_existent, None);

        let artifact_data = b"binary-artifact-output";
        client.put_artifact("art_01", artifact_data).unwrap();
        let fetched_art = client.get_artifact("art_01").unwrap();
        assert_eq!(fetched_art.as_deref(), Some(&artifact_data[..]));

        server.stop();
    }

    #[test]
    fn test_tcp_remote_cache_auth_failure() {
        let (server, addr, _server_handle) =
            start_test_server(Some("correct_token".to_string()), None);

        let client = TcpRemoteCacheClient::new(&addr, Some("wrong_token".to_string()));
        let err = client.ping().unwrap_err();
        assert!(matches!(err, RemoteCacheError::Protocol(_)));

        server.stop();
    }

    #[test]
    fn test_server_cas_dedups_identical_blobs() {
        let temp = tempdir().unwrap();
        let (server, addr, _server_handle) =
            start_test_server(None, Some(temp.path().to_path_buf()));

        let client = TcpRemoteCacheClient::new(&addr, None);
        let payload = b"identical-artifact-payload";
        client.put_artifact("key_a", payload).unwrap();
        client.put_artifact("key_b", payload).unwrap();

        assert_eq!(
            client.get_artifact("key_a").unwrap().as_deref(),
            Some(&payload[..])
        );
        assert_eq!(
            client.get_artifact("key_b").unwrap().as_deref(),
            Some(&payload[..])
        );

        let objects = temp.path().join("artifacts").join("objects");
        let count = std::fs::read_dir(&objects).unwrap().count();
        assert_eq!(count, 1, "identical content must be stored exactly once");

        server.stop();
    }

    #[test]
    fn test_server_artifacts_survive_restart() {
        let temp = tempdir().unwrap();
        let (server, addr, handle) = start_test_server(None, Some(temp.path().to_path_buf()));
        let client = TcpRemoteCacheClient::new(&addr, None);
        client.put_artifact("persist_me", b"durable").unwrap();
        client.put_fingerprint("fp_key", "fp_val").unwrap();
        server.stop();
        handle.join().unwrap();

        thread::sleep(Duration::from_millis(50));

        let mut restarted_opt = None;
        let mut handle2_opt = None;
        for _ in 0..10 {
            let s = RemoteCacheServer::new(&addr, None, Some(temp.path().to_path_buf()));
            if let Ok(h) = s.start_background() {
                restarted_opt = Some(s);
                handle2_opt = Some(h);
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        let restarted = restarted_opt.expect("failed to restart remote cache server");
        let handle2 = handle2_opt.unwrap();
        thread::sleep(Duration::from_millis(50));
        let client2 = TcpRemoteCacheClient::new(&addr, None);
        assert_eq!(
            client2.get_fingerprint("fp_key").unwrap().as_deref(),
            Some("fp_val")
        );
        assert_eq!(
            client2.get_artifact("persist_me").unwrap().as_deref(),
            Some(&b"durable"[..])
        );
        restarted.stop();
        handle2.join().unwrap();
    }

    #[derive(Debug)]
    struct ProducingExecutor {
        dir: std::path::PathBuf,
    }

    impl TaskExecutor for ProducingExecutor {
        fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
            let out = self.dir.join("out").join("app.bin");
            std::fs::create_dir_all(out.parent().unwrap()).unwrap();
            std::fs::write(&out, b"produced-binary").unwrap();
            Ok(TaskOutcome::executed(task))
        }
    }

    #[test]
    fn test_artifact_save_and_restore_roundtrip() {
        use forge_executor::{CommandSpec, Task};

        let temp = tempdir().unwrap();
        let remote = InMemoryRemoteCache::new();
        let local = LocalCache::new(temp.path().join("cache")).unwrap();
        let composite = CompositeCache::new(local, Some(Box::new(remote)));

        let producing = ProducingExecutor {
            dir: temp.path().to_path_buf(),
        };
        let caching = CompositeCachingExecutor::new(producing, composite);

        let spec = CommandSpec::new("true").cwd(temp.path());
        let task = Task::new("produce", "produce", spec)
            .with_cache(CacheEntry {
                key: "task/produce".to_string(),
                fingerprint: "fp-1".to_string(),
            })
            .with_artifacts(vec![std::path::PathBuf::from("out")]);

        let first = caching.execute(&task).unwrap();
        assert_eq!(first.status, TaskStatus::Executed);
        assert_eq!(
            std::fs::read(temp.path().join("out/app.bin")).unwrap(),
            b"produced-binary"
        );

        std::fs::remove_dir_all(temp.path().join("out")).unwrap();
        let second = caching.execute(&task).unwrap();
        assert_eq!(second.status, TaskStatus::Cached);
        assert_eq!(
            std::fs::read(temp.path().join("out/app.bin")).unwrap(),
            b"produced-binary",
            "a cached task must restore its artifacts before dependents run"
        );
    }

    #[test]
    fn test_tasks_without_artifacts_skip_store_and_restore() {
        use forge_executor::{CommandSpec, Task};

        let temp = tempdir().unwrap();
        let remote = InMemoryRemoteCache::new();
        let local = LocalCache::new(temp.path().join("cache")).unwrap();
        let composite = CompositeCache::new(local, Some(Box::new(remote)));

        let caching = CompositeCachingExecutor::new(
            ProducingExecutor {
                dir: temp.path().to_path_buf(),
            },
            composite,
        );

        let spec = CommandSpec::new("true").cwd(temp.path());
        let task = Task::new("plain", "plain", spec).with_cache(CacheEntry {
            key: "task/plain".to_string(),
            fingerprint: "fp-1".to_string(),
        });

        let first = caching.execute(&task).unwrap();
        assert_eq!(first.status, TaskStatus::Executed);
        let second = caching.execute(&task).unwrap();
        assert_eq!(second.status, TaskStatus::Cached);
    }
}
