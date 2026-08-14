#![forbid(unsafe_code)]

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use forge_executor::{CacheEntry, ExecutorError, Task, TaskExecutor, TaskOutcome, TaskStatus};
use serde::{Deserialize, Serialize};

// CAS-related types for future async integration
#[allow(dead_code)]
pub use forge_cas::{CasStorage, CasStorageConfig, Artifact, ArtifactHash};

#[derive(Debug)]
pub enum CacheError {
    NoHomeDir,

    Init { path: PathBuf, source: io::Error },

    Read { key: String, source: io::Error },

    Write { key: String, source: io::Error },
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHomeDir => write!(
                f,
                "cannot determine the user home directory (set HOME or USERPROFILE)"
            ),
            Self::Init { path, source } => {
                write!(
                    f,
                    "cannot initialize cache at `{}`: {source}",
                    path.display()
                )
            }
            Self::Read { key, source } => {
                write!(f, "cannot read cache record for key `{key}`: {source}")
            }
            Self::Write { key, source } => {
                write!(f, "cannot write cache record for key `{key}`: {source}")
            }
        }
    }
}

impl std::error::Error for CacheError {}

#[derive(Debug, Serialize, Deserialize)]
struct FingerprintRecord {
    fingerprint: String,

    stored_at: u64,

    #[serde(default)]
    artifact_hash: Option<String>,
}

/// One fingerprint record on disk, as reported by `LocalCache::records`.
#[derive(Debug, Clone)]
pub struct CacheRecord {
    pub key: String,
    pub fingerprint: String,
    pub stored_at: u64,
    pub path: PathBuf,
    pub size: u64,
}

/// A snapshot of what the cache currently occupies on disk.
#[derive(Debug, Clone, Default)]
pub struct CacheDiskStats {
    pub record_count: u64,
    pub fingerprints_bytes: u64,
    pub object_count: u64,
    pub objects_bytes: u64,
    pub total_bytes: u64,
}

/// What `LocalCache::prune` removed.
#[derive(Debug, Clone, Default)]
pub struct PruneReport {
    pub removed_records: u64,
    pub removed_objects: u64,
    pub freed_bytes: u64,
}

#[derive(Debug, Default)]
pub struct CacheStats {
    hits: AtomicU64,
    misses: AtomicU64,
    errors: AtomicU64,
}

impl CacheStats {
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::SeqCst);
    }

    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::SeqCst)
    }

    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::SeqCst)
    }

    pub fn errors(&self) -> u64 {
        self.errors.load(Ordering::SeqCst)
    }
}

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct LocalCache {
    root: PathBuf,
    stats: Arc<CacheStats>,
    cas_enabled: bool,
}

impl LocalCache {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, CacheError> {
        let root = root.into();
        fs::create_dir_all(root.join("metadata").join("fingerprints")).map_err(|source| {
            CacheError::Init {
                path: root.clone(),
                source,
            }
        })?;

        let _ = fs::create_dir_all(root.join("objects"));
        let _ = fs::create_dir_all(root.join("artifacts"));
        
        Ok(Self {
            root,
            stats: Arc::new(CacheStats::default()),
            cas_enabled: false,
        })
    }
    
    pub fn with_cas(mut self, enable: bool) -> Self {
        self.cas_enabled = enable;
        self
    }
    
    pub fn is_cas_enabled(&self) -> bool {
        self.cas_enabled
    }
    
    pub fn cas_path(&self) -> PathBuf {
        self.root.join("cas")
    }

    pub fn default_location() -> Result<Self, CacheError> {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .ok_or(CacheError::NoHomeDir)?;
        Self::new(PathBuf::from(home).join(".forge").join("cache"))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    pub fn matches(&self, key: &str, fingerprint: &str) -> bool {
        match self.read_record(key) {
            Ok(Some(record)) if record.fingerprint == fingerprint => {
                self.stats.record_hit();
                true
            }
            _ => {
                self.stats.record_miss();
                false
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.read_record(key).ok().flatten().map(|r| r.fingerprint)
    }

    pub fn put(&self, key: &str, fingerprint: &str) -> Result<(), CacheError> {
        self.put_with_artifact(key, fingerprint, None)
    }

    pub fn put_with_artifact(
        &self,
        key: &str,
        fingerprint: &str,
        artifact_hash: Option<String>,
    ) -> Result<(), CacheError> {
        let path = self.fingerprint_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| CacheError::Write {
                key: key.to_string(),
                source,
            })?;
        }
        let record = FingerprintRecord {
            fingerprint: fingerprint.to_string(),
            stored_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            artifact_hash,
        };
        let payload = serde_json::to_vec(&record).expect("a fingerprint record always serializes");
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, payload).map_err(|source| CacheError::Write {
            key: key.to_string(),
            source,
        })?;
        fs::rename(&tmp, &path).map_err(|source| CacheError::Write {
            key: key.to_string(),
            source,
        })
    }

    pub fn artifact_hash(&self, key: &str) -> Option<String> {
        self.read_record(key)
            .ok()
            .flatten()
            .and_then(|r| r.artifact_hash)
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.root.join("objects")
    }

    pub fn put_object(&self, hash: &str, bytes: &[u8]) -> Result<(), CacheError> {
        let path = self.objects_dir().join(hash);
        let tmp = self.objects_dir().join(format!("{hash}.tmp"));
        fs::write(&tmp, bytes).map_err(|source| CacheError::Write {
            key: hash.to_string(),
            source,
        })?;
        fs::rename(&tmp, &path).map_err(|source| CacheError::Write {
            key: hash.to_string(),
            source,
        })
    }

    pub fn get_object(&self, hash: &str) -> Option<Vec<u8>> {
        fs::read(self.objects_dir().join(hash)).ok()
    }

    fn fingerprint_path(&self, key: &str) -> PathBuf {
        self.root
            .join("metadata")
            .join("fingerprints")
            .join(format!("{key}.json"))
    }

    fn read_record(&self, key: &str) -> Result<Option<FingerprintRecord>, CacheError> {
        let path = self.fingerprint_path(key);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(CacheError::Read {
                    key: key.to_string(),
                    source,
                });
            }
        };

        Ok(serde_json::from_slice(&bytes).ok())
    }

    /// Enumerates every fingerprint record on disk, skipping corrupt files.
    pub fn records(&self) -> Vec<CacheRecord> {
        let mut out = Vec::new();
        let root = self.root.join("metadata").join("fingerprints");
        let Ok(entries) = walk_files(&root) else {
            return out;
        };
        for path in entries {
            let Some(extension) = path.extension().map(|e| e.to_string_lossy().to_string())
            else {
                continue;
            };
            if extension != "json" {
                continue;
            }
            let Ok(bytes) = fs::read(&path) else {
                continue;
            };
            let Ok(record) = serde_json::from_slice::<FingerprintRecord>(&bytes) else {
                continue;
            };
            let key = path
                .strip_prefix(&root)
                .ok()
                .and_then(|relative| {
                    relative
                        .to_str()
                        .map(|text| text.strip_suffix(".json").unwrap_or(text))
                })
                .map(|text| text.replace('\\', "/"))
                .unwrap_or_else(|| path.to_string_lossy().into_owned());
            out.push(CacheRecord {
                key,
                fingerprint: record.fingerprint,
                stored_at: record.stored_at,
                size: bytes.len() as u64,
                path,
            });
        }
        out
    }

    /// Byte-level snapshot of what the cache occupies on disk.
    pub fn disk_stats(&self) -> CacheDiskStats {
        let records = self.records();
        let objects: Vec<PathBuf> = walk_files(&self.objects_dir()).unwrap_or_default();
        let objects_bytes = objects
            .iter()
            .filter_map(|p| fs::metadata(p).ok())
            .map(|m| m.len())
            .sum::<u64>();
        CacheDiskStats {
            record_count: records.len() as u64,
            fingerprints_bytes: records.iter().map(|r| r.size).sum(),
            object_count: objects.len() as u64,
            objects_bytes,
            total_bytes: records.iter().map(|r| r.size).sum::<u64>() + objects_bytes,
        }
    }

    /// Removes records older than `older_than` and, when the cache still
    /// exceeds `max_size` bytes, deletes the oldest records/objects first.
    pub fn prune(
        &self,
        older_than: Option<Duration>,
        max_size: Option<u64>,
    ) -> Result<PruneReport, CacheError> {
        let mut report = PruneReport::default();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut records = self.records();
        records.sort_by_key(|r| r.stored_at);

        let mut objects = walk_files(&self.objects_dir()).unwrap_or_default();
        objects.sort_by_key(|p| {
            fs::metadata(p)
                .and_then(|m| m.modified())
                .ok()
                .unwrap_or(UNIX_EPOCH)
        });

        let object_bytes: Vec<u64> = objects
            .iter()
            .map(|p| fs::metadata(p).map(|m| m.len()).unwrap_or(0))
            .collect();
        let mut total =
            records.iter().map(|r| r.size).sum::<u64>() + object_bytes.iter().sum::<u64>();
        let max = max_size.unwrap_or(u64::MAX);

        for record in records {
            let too_old = older_than
                .map(|age| now.saturating_sub(record.stored_at) >= age.as_secs())
                .unwrap_or(false);
            if (too_old || total > max) && fs::remove_file(&record.path).is_ok() {
                report.removed_records += 1;
                report.freed_bytes += record.size;
                total = total.saturating_sub(record.size);
            }
        }

        for (path, size) in objects.into_iter().zip(object_bytes) {
            if total > max && fs::remove_file(&path).is_ok() {
                report.removed_objects += 1;
                report.freed_bytes += size;
                total = total.saturating_sub(size);
            }
        }

        Ok(report)
    }
}

fn walk_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    Ok(out)
}

#[derive(Debug)]
pub struct CachingExecutor<I> {
    inner: I,
    cache: LocalCache,
}

impl<I: TaskExecutor> CachingExecutor<I> {
    pub fn new(inner: I, cache: LocalCache) -> Self {
        Self { inner, cache }
    }

    pub fn cache(&self) -> &LocalCache {
        &self.cache
    }

    pub fn into_inner(self) -> I {
        self.inner
    }
}

impl<I: TaskExecutor> TaskExecutor for CachingExecutor<I> {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        if let Some(CacheEntry { key, fingerprint }) = &task.cache {
            if self.cache.matches(key, fingerprint) {
                // Note: CAS artifact restoration will be handled by the caller/CLI
                // for now, we just return cached status
                return Ok(TaskOutcome::cached(task));
            }
        }
        let outcome = self.inner.execute(task)?;
        if outcome.status == TaskStatus::Executed {
            if let Some(CacheEntry { key, fingerprint }) = &task.cache {
                if let Err(_error) = self.cache.put(key, fingerprint) {
                    self.cache.stats().record_error();
                }
                
                // Note: CAS artifact storage will be handled by the caller/CLI
                // for now, we skip async operations in sync context
            }
        }
        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache() -> (LocalCache, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = LocalCache::new(dir.path()).expect("cache init");
        (cache, dir)
    }

    #[test]
    fn put_get_roundtrip() {
        let (cache, _dir) = cache();
        cache.put("a", "fp-1").unwrap();
        assert_eq!(cache.get("a").as_deref(), Some("fp-1"));
    }

    #[test]
    fn matches_distinguishes_hits_and_misses() {
        let (cache, _dir) = cache();
        cache.put("a", "fp-1").unwrap();
        assert!(cache.matches("a", "fp-1"));
        assert!(!cache.matches("a", "fp-2"));
        assert!(!cache.matches("b", "fp-1"));
        assert_eq!(cache.stats().hits(), 1);
        assert_eq!(cache.stats().misses(), 2);
    }

    #[test]
    fn missing_record_is_a_miss() {
        let (cache, _dir) = cache();
        assert!(!cache.matches("nope", "x"));
        assert!(cache.get("nope").is_none());
    }

    #[test]
    fn corrupt_record_is_a_miss_and_does_not_panic() {
        let (cache, _dir) = cache();
        let path = cache.fingerprint_path("bad");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"this is not json").unwrap();
        assert!(!cache.matches("bad", "x"));
        assert!(cache.get("bad").is_none());
    }

    #[test]
    fn put_replaces_previous_value() {
        let (cache, _dir) = cache();
        cache.put("a", "v1").unwrap();
        cache.put("a", "v2").unwrap();
        assert!(cache.matches("a", "v2"));
        assert!(!cache.matches("a", "v1"));
    }

    #[test]
    fn concurrent_puts_are_safe() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = LocalCache::new(dir.path()).unwrap();

        std::thread::scope(|scope| {
            let cache = &cache;
            for thread in 0..8 {
                scope.spawn(move || {
                    for i in 0..50 {
                        let key = format!("t{thread}-k{i}");
                        let fp = format!("fp-{thread}-{i}");
                        cache.put(&key, &fp).unwrap();
                    }
                });
            }
        });
        for thread in 0..8 {
            for i in 0..50 {
                let key = format!("t{thread}-k{i}");
                let fp = format!("fp-{thread}-{i}");
                assert!(cache.matches(&key, &fp), "missing record for {key}");
            }
        }
    }

    #[test]
    fn default_location_requires_a_home_directory() {
        let original = (std::env::var_os("HOME"), std::env::var_os("USERPROFILE"));

        unsafe { std::env::remove_var("HOME") };
        unsafe { std::env::remove_var("USERPROFILE") };
        assert!(matches!(
            LocalCache::default_location(),
            Err(CacheError::NoHomeDir)
        ));

        if let Some(home) = original.0 {
            unsafe { std::env::set_var("HOME", home) };
        }
        if let Some(profile) = original.1 {
            unsafe { std::env::set_var("USERPROFILE", profile) };
        }
    }

    #[test]
    fn objects_roundtrip_and_dedup() {
        let (cache, _dir) = cache();
        cache.put_object("abc", b"payload").unwrap();
        assert_eq!(cache.get_object("abc").as_deref(), Some(&b"payload"[..]));
        assert!(cache.get_object("nope").is_none());
    }

    #[test]
    fn artifact_hash_is_persisted_in_the_record() {
        let (cache, _dir) = cache();
        cache.put_with_artifact("k", "fp", Some("blob-hash".to_string())).unwrap();
        assert_eq!(cache.artifact_hash("k").as_deref(), Some("blob-hash"));
        assert_eq!(cache.get("k").as_deref(), Some("fp"));
        assert!(cache.matches("k", "fp"));

        cache.put("k", "fp2").unwrap();
        assert_eq!(cache.artifact_hash("k"), None);
    }

    #[test]
    fn records_and_disk_stats_walk_nested_keys() {
        let (cache, _dir) = cache();
        cache.put("v1/ns/build/level/a", "fp-a").unwrap();
        cache.put("v1/ns/build/level/b", "fp-b").unwrap();
        cache.put_object("obj-1", b"some bytes").unwrap();

        let stats = cache.disk_stats();
        assert_eq!(stats.record_count, 2);
        assert_eq!(stats.object_count, 1);
        assert!(stats.fingerprints_bytes > 0);
        assert_eq!(stats.total_bytes, stats.fingerprints_bytes + stats.objects_bytes);

        let records = cache.records();
        assert_eq!(records.len(), 2);
        let keys: Vec<&str> = records.iter().map(|r| r.key.as_str()).collect();
        assert!(keys.contains(&"v1/ns/build/level/a"));
        assert!(records.iter().all(|r| r.size > 0));
    }

    #[test]
    fn prune_removes_old_records() {
        let (cache, _dir) = cache();
        cache.put("old", "fp-old").unwrap();
        cache.put("fresh", "fp-fresh").unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let old_record = serde_json::json!({
            "fingerprint": "fp-old",
            "stored_at": now - 86_400,
        });
        let path = cache.fingerprint_path("old");
        fs::write(&path, serde_json::to_vec(&old_record).unwrap()).unwrap();

        let report = cache.prune(Some(Duration::from_secs(60 * 60)), None).unwrap();
        assert_eq!(report.removed_records, 1);
        assert_eq!(report.removed_objects, 0);
        assert!(report.freed_bytes > 0);
        assert!(cache.get("old").is_none());
        assert!(cache.get("fresh").is_some());
    }

    #[test]
    fn prune_enforces_max_size_oldest_first() {
        let (cache, _dir) = cache();
        cache.put("a", "fp-a").unwrap();
        cache.put_object("blob", vec![0u8; 512].as_slice()).unwrap();

        let report = cache.prune(None, Some(1)).unwrap();
        assert!(report.removed_records >= 1);
        assert!(report.freed_bytes > 0);
        assert_eq!(cache.disk_stats().total_bytes, 0);
    }

    #[test]
    fn prune_with_no_pressure_removes_nothing() {
        let (cache, _dir) = cache();
        cache.put("a", "fp-a").unwrap();
        cache.put_object("blob", b"x").unwrap();
        let report = cache.prune(None, None).unwrap();
        assert_eq!(report.removed_records, 0);
        assert_eq!(report.removed_objects, 0);
        assert_eq!(cache.disk_stats().record_count, 1);
    }
}
