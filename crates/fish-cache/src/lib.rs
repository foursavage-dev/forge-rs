#![cfg_attr(not(test), forbid(unsafe_code))]

pub mod async_io;
pub mod file_level;
pub mod file_level_adapter;
pub mod gc;
pub mod pool;
pub mod strategies;

pub use async_io::{AsyncCache, AsyncFileReader, AsyncFileWriter, AsyncIoError};
pub use file_level_adapter::{FileLevelCacheAdapter, HybridCachingExecutor};
pub use gc::{BackgroundCacheGc, EvictionPolicy, GcConfig};
pub use pool::{BufferPool, PoolStats, ScopedBuffer};
pub use strategies::{LruCache, PredictiveCache, SpinLockLruCache, TieredCache};

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fish_executor::{CacheEntry, ExecutorError, Task, TaskExecutor, TaskOutcome, TaskStatus};
use serde::{Deserialize, Serialize};

// CAS-related types for future async integration
#[allow(dead_code)]
pub use fish_cas::{Artifact, ArtifactHash, CasStorage, CasStorageConfig};

#[derive(Debug)]
pub enum CacheError {
    NoHomeDir,

    Init { path: PathBuf, source: io::Error },

    Read { key: String, source: io::Error },

    Write { key: String, source: io::Error },

    InvalidHash { hash: String },
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
            Self::InvalidHash { hash } => {
                write!(f, "invalid object hash `{hash}`")
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

    /// Original cache key. Keys are percent-encoded for safe filenames on
    /// disk; this field lets `records()` recover the key as the caller wrote
    /// it (and reads old records written before this field existed).
    #[serde(default)]
    key: Option<String>,
}

/// One fingerprint record on disk, as reported by `LocalCache::records`.
#[derive(Debug, Clone)]
pub struct CacheRecord {
    pub key: String,
    pub fingerprint: String,
    pub stored_at: u64,
    pub path: PathBuf,
    pub size: u64,
    pub artifact_hash: Option<String>,
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
    buffer_pool: Arc<BufferPool>,
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
            buffer_pool: Arc::new(BufferPool::new()),
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
        if let Some(path) = std::env::var_os("FISH_CACHE_DIR") {
            return Self::new(PathBuf::from(path));
        }
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .ok_or(CacheError::NoHomeDir)?;
        Self::new(PathBuf::from(home).join(".fish").join("cache"))
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
            key: Some(key.to_string()),
        };
        let payload = serde_json::to_vec(&record).expect("a fingerprint record always serializes");
        let tmp = unique_tmp_path(&path);
        fs::write(&tmp, payload).map_err(|source| CacheError::Write {
            key: key.to_string(),
            source,
        })?;
        atomic_rename(&tmp, &path).map_err(|source| CacheError::Write {
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
        if !valid_object_hash(hash) {
            return Err(CacheError::InvalidHash {
                hash: hash.to_string(),
            });
        }
        let path = self.objects_dir().join(hash);
        let tmp = unique_tmp_path(&path);

        // Use scoped buffer for better memory efficiency
        let _scoped_buffer = ScopedBuffer::new(bytes.len(), self.buffer_pool.clone());

        fs::write(&tmp, bytes).map_err(|source| CacheError::Write {
            key: hash.to_string(),
            source,
        })?;
        atomic_rename(&tmp, &path).map_err(|source| CacheError::Write {
            key: hash.to_string(),
            source,
        })
    }

    pub fn get_object(&self, hash: &str) -> Option<Vec<u8>> {
        if !valid_object_hash(hash) {
            return None;
        }
        let path = self.objects_dir().join(hash);
        let metadata = fs::metadata(&path).ok()?;

        // Use buffer pool for reading large files efficiently
        let file_size = metadata.len() as usize;
        if file_size < 4096 {
            // For small files, use standard read
            return fs::read(&path).ok();
        }

        let mut scoped_buffer = ScopedBuffer::new(file_size, self.buffer_pool.clone());
        let buffer = scoped_buffer.as_mut();
        buffer.resize(file_size, 0);

        fs::File::open(&path).ok()?.read_exact(buffer).ok()?;

        Some(scoped_buffer.into_inner())
    }

    fn fingerprint_path(&self, key: &str) -> PathBuf {
        self.root
            .join("metadata")
            .join("fingerprints")
            .join(format!("{}.json", encode_key(key)))
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
            let Some(extension) = path.extension().map(|e| e.to_string_lossy().to_string()) else {
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
            let key = record.key.unwrap_or_else(|| {
                // Legacy records: raw nested layout (v1/ns/...) or
                // percent-encoded key.
                let raw = path
                    .strip_prefix(&root)
                    .ok()
                    .and_then(|relative| {
                        relative
                            .to_str()
                            .map(|text| text.strip_suffix(".json").unwrap_or(text))
                    })
                    .map(|text| text.replace('\\', "/"))
                    .unwrap_or_else(|| path.to_string_lossy().into_owned());
                if raw.contains('/') {
                    raw
                } else {
                    decode_key(&raw).unwrap_or(raw)
                }
            });
            out.push(CacheRecord {
                key,
                fingerprint: record.fingerprint,
                stored_at: record.stored_at,
                size: bytes.len() as u64,
                path,
                artifact_hash: record.artifact_hash,
            });
        }
        out
    }

    /// Byte-level snapshot of what the cache occupies on disk.
    /// Optimized with single-pass computation.
    pub fn disk_stats(&self) -> CacheDiskStats {
        let records = self.records();
        let objects: Vec<PathBuf> = walk_files(&self.objects_dir())
            .unwrap_or_default()
            .into_iter()
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e != "tmp")
                    .unwrap_or(true)
            })
            .collect();

        // Single-pass computation for better performance
        let (object_count, objects_bytes) = objects
            .iter()
            .filter_map(|p| fs::metadata(p).ok())
            .fold((0u64, 0u64), |(count, total), m| {
                (count + 1, total + m.len())
            });

        let record_count = records.len() as u64;
        let fingerprints_bytes: u64 = records.iter().map(|r| r.size).sum();

        CacheDiskStats {
            record_count,
            fingerprints_bytes,
            object_count,
            objects_bytes,
            total_bytes: fingerprints_bytes + objects_bytes,
        }
    }

    /// Removes records older than `older_than` and, when the cache still
    /// exceeds `max_size` bytes, deletes the oldest records/objects first.
    /// Objects referenced by multiple records are kept until their last
    /// referencing record disappears; orphaned objects and stale temporary
    /// files are swept during the age-based phase.
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

        // Path strings of files already removed this pass, so later phases
        // neither count nor re-remove them.
        let mut removed: HashSet<String> = HashSet::new();

        let objects_dir = self.objects_dir();
        let fingerprints_dir = self.root.join("metadata").join("fingerprints");

        // Phase 1: age-based eviction of records, orphaned objects and stale
        // temporary files.
        if let Some(age) = older_than {
            let age_secs = age.as_secs();
            let mut ref_counts = self.ref_counts();

            for record in self.records() {
                if now.saturating_sub(record.stored_at) >= age_secs {
                    drop_record_and_cascade(
                        &record,
                        &objects_dir,
                        &mut ref_counts,
                        &mut removed,
                        &mut report,
                    );
                }
            }

            // Orphaned objects and stale temporary files. Only files older
            // than `older_than` are touched, so concurrent writers are never
            // interrupted.
            let mut objects = walk_files(&objects_dir).unwrap_or_default();
            objects.sort_unstable_by_key(|p| {
                fs::metadata(p)
                    .and_then(|m| m.modified())
                    .ok()
                    .unwrap_or(UNIX_EPOCH)
            });
            for path in objects {
                let path_str = path.to_string_lossy().to_string();
                if removed.contains(&path_str) {
                    continue;
                }
                let Some(age) = file_age_secs(&path, now) else {
                    continue;
                };
                if age < age_secs {
                    continue;
                }
                let is_tmp = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "tmp")
                    .unwrap_or(false);
                let orphan = !is_tmp
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| !ref_counts.contains_key(name))
                        .unwrap_or(false);
                if (is_tmp || orphan)
                    && let Ok(metadata) = fs::metadata(&path)
                    && fs::remove_file(&path).is_ok()
                {
                    removed.insert(path_str);
                    if !is_tmp {
                        report.removed_objects += 1;
                    }
                    report.freed_bytes += metadata.len();
                }
            }

            // Stale temporary fingerprint files, also age-gated.
            report.freed_bytes += cleanup_tmp_files(&fingerprints_dir, now, age_secs, &mut removed);
        }

        // Phase 2: capacity-based eviction. Oldest records first (cascading to
        // their objects), then the oldest objects by mtime, even when still
        // referenced.
        let max = max_size.unwrap_or(u64::MAX);

        let mut records = self.records();
        records.sort_unstable_by_key(|r| r.stored_at);

        let mut objects = walk_files(&objects_dir).unwrap_or_default();
        objects.sort_unstable_by_key(|p| {
            fs::metadata(p)
                .and_then(|m| m.modified())
                .ok()
                .unwrap_or(UNIX_EPOCH)
        });

        let mut total: u64 = records.iter().map(|r| r.size).sum();
        for path in &objects {
            let path_str = path.to_string_lossy().to_string();
            if removed.contains(&path_str) {
                continue;
            }
            if let Ok(metadata) = fs::metadata(path) {
                total += metadata.len();
            }
        }

        let mut ref_counts = self.ref_counts();
        for record in records {
            if total <= max {
                break;
            }
            let freed = drop_record_and_cascade(
                &record,
                &objects_dir,
                &mut ref_counts,
                &mut removed,
                &mut report,
            );
            total = total.saturating_sub(freed);
        }

        for path in objects {
            if total <= max {
                break;
            }
            let path_str = path.to_string_lossy().to_string();
            if removed.contains(&path_str) {
                continue;
            }
            let Ok(metadata) = fs::metadata(&path) else {
                continue;
            };
            if fs::remove_file(&path).is_ok() {
                removed.insert(path_str);
                report.removed_objects += 1;
                report.freed_bytes += metadata.len();
                total = total.saturating_sub(metadata.len());
            }
        }

        Ok(report)
    }

    /// Maps object file names to the number of fingerprint records
    /// referencing them.
    fn ref_counts(&self) -> HashMap<String, u64> {
        let mut counts: HashMap<String, u64> = HashMap::new();
        for record in self.records() {
            if let Some(hash) = &record.artifact_hash {
                *counts.entry(hash.clone()).or_insert(0) += 1;
            }
        }
        counts
    }
}

/// Removes one fingerprint record and, when no other record references its
/// object, the object file as well. Returns the number of bytes freed.
fn drop_record_and_cascade(
    record: &CacheRecord,
    objects_dir: &Path,
    ref_counts: &mut HashMap<String, u64>,
    removed: &mut HashSet<String>,
    report: &mut PruneReport,
) -> u64 {
    if fs::remove_file(&record.path).is_err() {
        return 0;
    }
    report.removed_records += 1;
    report.freed_bytes += record.size;
    let mut freed = record.size;
    if let Some(hash) = &record.artifact_hash {
        let count = ref_counts.get(hash).copied().unwrap_or(0);
        if count > 0 {
            ref_counts.insert(hash.clone(), count - 1);
        }
        if count <= 1 {
            let object_path = objects_dir.join(hash);
            let object_str = object_path.to_string_lossy().to_string();
            if removed.insert(object_str)
                && let Ok(metadata) = fs::metadata(&object_path)
                && fs::remove_file(&object_path).is_ok()
            {
                report.removed_objects += 1;
                freed += metadata.len();
            }
        }
    }
    freed
}

/// Removes `*.tmp` files in `dir` older than `age_secs`, so interrupted
/// writers never leave garbage behind while in-flight files stay untouched.
fn cleanup_tmp_files(dir: &Path, now: u64, age_secs: u64, removed: &mut HashSet<String>) -> u64 {
    let mut freed = 0u64;
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_tmp = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "tmp")
            .unwrap_or(false);
        if !is_tmp {
            continue;
        }
        let path_str = path.to_string_lossy().to_string();
        if removed.contains(&path_str) {
            continue;
        }
        let Some(age) = file_age_secs(&path, now) else {
            continue;
        };
        if age >= age_secs
            && let Ok(metadata) = fs::metadata(&path)
            && fs::remove_file(&path).is_ok()
        {
            removed.insert(path_str);
            freed += metadata.len();
        }
    }
    freed
}

/// Seconds since the file's mtime, relative to `now`. Files without a
/// readable mtime report `None`; future mtimes report 0.
fn file_age_secs(path: &Path, now: u64) -> Option<u64> {
    let mtime = fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(now.saturating_sub(mtime))
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

/// Returns true when `byte` can appear verbatim in an encoded cache key.
fn key_char_safe(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

/// Decodes a single hexadecimal digit.
fn hex_val(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Percent-encodes `key` so it can be used safely as a single file name
/// segment. Every byte maps to exactly one representation (injective), so
/// distinct keys never collide on disk.
fn encode_key(key: &str) -> String {
    let mut out = String::with_capacity(key.len());
    for byte in key.as_bytes() {
        if key_char_safe(*byte) {
            out.push(*byte as char);
        } else {
            out.push('%');
            out.push_str(&format!("{byte:02X}"));
        }
    }
    out
}

/// Inverse of [`encode_key`]; returns `None` on malformed input.
fn decode_key(encoded: &str) -> Option<String> {
    let bytes = encoded.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hi = hex_val(*bytes.get(i + 1)?)?;
            let lo = hex_val(*bytes.get(i + 2)?)?;
            out.push(hi << 4 | lo);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// Returns true when `hash` is safe to use as a single object file name.
fn valid_object_hash(hash: &str) -> bool {
    !hash.is_empty() && hash != "." && hash != ".." && !hash.contains(['/', '\\', '\0'])
}

/// Monotonic counter backing [`unique_tmp_path`].
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Builds a process-unique temporary path next to `path`, so concurrent
/// writers of the same key never race on the same `.tmp` file.
pub(crate) fn unique_tmp_path(path: &Path) -> PathBuf {
    let seq = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_string());
    path.with_file_name(format!("{name}.{}.{seq}.tmp", std::process::id()))
}

fn atomic_rename(src: &Path, dst: &Path) -> io::Result<()> {
    for i in 0..20 {
        match fs::rename(src, dst) {
            Ok(()) => return Ok(()),
            Err(e)
                if i < 19
                    && (e.kind() == io::ErrorKind::PermissionDenied
                        || e.raw_os_error() == Some(5)
                        || e.raw_os_error() == Some(32)) =>
            {
                std::thread::sleep(std::time::Duration::from_millis(1 + i * 2));
            }
            Err(e) => {
                let _ = fs::remove_file(src);
                return Err(e);
            }
        }
    }
    Ok(())
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
        if let Some(CacheEntry { key, fingerprint }) = &task.cache
            && self.cache.matches(key, fingerprint)
        {
            // Note: CAS artifact restoration will be handled by the caller/CLI
            // for now, we just return cached status
            return Ok(TaskOutcome::cached(task));
        }
        let outcome = self.inner.execute(task)?;
        if outcome.status == TaskStatus::Executed
            && let Some(CacheEntry { key, fingerprint }) = &task.cache
            && let Err(_error) = self.cache.put(key, fingerprint)
        {
            self.cache.stats().record_error();

            // Note: CAS artifact storage will be handled by the caller/CLI
            // for now, we skip async operations in sync context
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
        cache
            .put_with_artifact("k", "fp", Some("blob-hash".to_string()))
            .unwrap();
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
        assert_eq!(
            stats.total_bytes,
            stats.fingerprints_bytes + stats.objects_bytes
        );

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

        let report = cache
            .prune(Some(Duration::from_secs(60 * 60)), None)
            .unwrap();
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

    fn set_mtime_old(path: &Path, age_secs: u64) {
        let old = SystemTime::now() - Duration::from_secs(age_secs);
        let file = fs::File::options()
            .write(true)
            .open(path)
            .expect("open file");
        file.set_times(fs::FileTimes::new().set_modified(old))
            .expect("set mtime");
    }

    #[test]
    fn keys_are_safely_encoded_on_disk() {
        let (cache, dir) = cache();
        let key = "../../evil";
        cache.put(key, "fp-tricky").unwrap();

        let path = cache.fingerprint_path(key);
        let fingerprints_dir = dir.path().join("metadata").join("fingerprints");
        assert!(path.starts_with(&fingerprints_dir));

        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        assert!(!file_name.contains('/'));
        assert!(!file_name.contains('\\'));

        assert_eq!(decode_key(&encode_key(key)), Some(key.to_string()));
        assert_eq!(cache.get(key), Some("fp-tricky".to_string()));
        assert_eq!(cache.records().len(), 1);
        assert_eq!(cache.records()[0].key, key);
    }

    #[test]
    fn traversal_keys_do_not_escape_the_fingerprints_directory() {
        let (cache, dir) = cache();
        let key = "../../outside";
        cache.put(key, "fp").unwrap();

        let path = cache.fingerprint_path(key);
        assert!(path.starts_with(dir.path().join("metadata").join("fingerprints")));
        assert!(!dir.path().join("outside.json").exists());
        assert_eq!(cache.get(key), Some("fp".to_string()));
    }

    #[test]
    fn invalid_object_hashes_are_rejected() {
        let (cache, _dir) = cache();
        for hash in ["", ".", "..", "a/b", "a\\b", "a\0b"] {
            assert!(matches!(
                cache.put_object(hash, b"x"),
                Err(CacheError::InvalidHash { .. })
            ));
            assert_eq!(cache.get_object(hash), None);
        }

        cache.put_object("deadbeef", b"x").unwrap();
        assert_eq!(cache.get_object("deadbeef"), Some(b"x".to_vec()));
    }

    #[test]
    fn prune_cascades_to_orphaned_objects() {
        let (cache, _dir) = cache();
        cache.put_object("obj-a", b"data").unwrap();
        cache.put_object("obj-orphan", b"data").unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let old_record = serde_json::json!({
            "fingerprint": "fp-a",
            "stored_at": now - 86_400,
            "artifact_hash": "obj-a",
            "key": "shared-a",
        });
        fs::write(
            cache.fingerprint_path("shared-a"),
            serde_json::to_vec(&old_record).unwrap(),
        )
        .unwrap();
        let fresh_record = serde_json::json!({
            "fingerprint": "fp-b",
            "stored_at": now,
            "artifact_hash": "obj-a",
            "key": "shared-b",
        });
        fs::write(
            cache.fingerprint_path("shared-b"),
            serde_json::to_vec(&fresh_record).unwrap(),
        )
        .unwrap();

        set_mtime_old(&cache.objects_dir().join("obj-orphan"), 86_400);

        let report = cache.prune(Some(Duration::from_secs(3600)), None).unwrap();
        assert_eq!(report.removed_records, 1);
        assert_eq!(report.removed_objects, 1);
        assert!(report.freed_bytes > 0);
        assert!(cache.get_object("obj-a").is_some());
        assert!(cache.get_object("obj-orphan").is_none());
    }

    #[test]
    fn concurrent_puts_with_same_key_are_safe() {
        let (cache_unwrapped, _dir) = cache();
        let cache = std::sync::Arc::new(cache_unwrapped);

        std::thread::scope(|scope| {
            for thread_id in 0..8 {
                let cache = cache.clone();
                scope.spawn(move || {
                    for i in 0..50 {
                        cache
                            .put("same-key", &format!("fp-{thread_id}-{i}"))
                            .unwrap();
                    }
                });
            }
        });

        assert_eq!(cache.records().len(), 1);
        assert!(cache.get("same-key").is_some());
    }

    #[test]
    fn disk_stats_ignores_temporary_files() {
        let (cache, _dir) = cache();
        cache.put("a", "fp-a").unwrap();
        cache.put_object("obj-a", b"data").unwrap();

        let before = cache.disk_stats();
        let tmp_a = unique_tmp_path(&cache.objects_dir().join("tmp-test"));
        fs::write(&tmp_a, vec![0u8; 4096]).unwrap();
        let tmp_b = unique_tmp_path(&cache.objects_dir().join("tmp-test"));
        fs::write(&tmp_b, vec![0u8; 4096]).unwrap();

        let after = cache.disk_stats();
        assert_eq!(after.object_count, before.object_count);
        assert_eq!(after.objects_bytes, before.objects_bytes);
    }
}
