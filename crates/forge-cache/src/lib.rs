use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use forge_executor::{CacheEntry, ExecutorError, Task, TaskExecutor, TaskOutcome, TaskStatus};
use serde::{Deserialize, Serialize};

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

#[derive(Debug)]
pub struct LocalCache {
    root: PathBuf,
    stats: CacheStats,
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
            stats: CacheStats::default(),
        })
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
                return Ok(TaskOutcome::cached(task));
            }
        }
        let outcome = self.inner.execute(task)?;
        if outcome.status == TaskStatus::Executed {
            if let Some(CacheEntry { key, fingerprint }) = &task.cache {
                if let Err(_error) = self.cache.put(key, fingerprint) {
                    self.cache.stats().record_error();
                }
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
}
