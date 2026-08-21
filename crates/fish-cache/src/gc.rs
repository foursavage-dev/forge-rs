use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvictionPolicy {
    Lru,
    Lfu,
    TtlOnly,
}

#[derive(Debug, Clone)]
pub struct GcConfig {
    pub max_size_bytes: u64,
    pub high_watermark_ratio: f64,
    pub low_watermark_ratio: f64,
    pub ttl: Duration,
    pub policy: EvictionPolicy,
    pub scan_interval: Duration,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 20 * 1024 * 1024 * 1024,
            high_watermark_ratio: 0.90,
            low_watermark_ratio: 0.70,
            ttl: Duration::from_secs(7 * 86400),
            policy: EvictionPolicy::Lru,
            scan_interval: Duration::from_secs(300),
        }
    }
}

pub struct BackgroundCacheGc {
    cache_root: PathBuf,
    config: GcConfig,
    running: Arc<AtomicBool>,
}

impl BackgroundCacheGc {
    pub fn new(cache_root: PathBuf, config: GcConfig) -> Self {
        Self {
            cache_root,
            config,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) -> Arc<AtomicBool> {
        self.running.store(true, Ordering::SeqCst);
        self.running.clone()
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn config(&self) -> &GcConfig {
        &self.config
    }

    pub fn scan_and_evict(&self) -> (usize, u64) {
        let mut removed_count = 0;
        let mut freed_bytes = 0;

        let now = SystemTime::now();

        if let Ok(entries) = std::fs::read_dir(&self.cache_root) {
            let mut file_entries = Vec::new();

            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(meta) = path.metadata()
                    && meta.is_file()
                {
                    let size = meta.len();
                    let mtime = meta.modified().unwrap_or(now);
                    let age = now.duration_since(mtime).unwrap_or_default();

                    if age >= self.config.ttl {
                        if std::fs::remove_file(&path).is_ok() {
                            removed_count += 1;
                            freed_bytes += size;
                        }
                    } else {
                        file_entries.push((path, size, mtime));
                    }
                }
            }

            let total_size: u64 = file_entries.iter().map(|(_, sz, _)| *sz).sum();
            let threshold =
                (self.config.max_size_bytes as f64 * self.config.high_watermark_ratio) as u64;
            let target_size =
                (self.config.max_size_bytes as f64 * self.config.low_watermark_ratio) as u64;

            if total_size > threshold {
                match self.config.policy {
                    EvictionPolicy::Lru => {
                        file_entries.sort_by_key(|(_, _, mtime)| *mtime);
                    }
                    EvictionPolicy::Lfu => {
                        file_entries.sort_by_key(|(_, sz, _)| *sz);
                    }
                    EvictionPolicy::TtlOnly => {}
                }

                let mut current_size = total_size;
                for (path, size, _) in file_entries {
                    if current_size <= target_size {
                        break;
                    }
                    if std::fs::remove_file(&path).is_ok() {
                        removed_count += 1;
                        freed_bytes += size;
                        current_size = current_size.saturating_sub(size);
                    }
                }
            }
        }

        (removed_count, freed_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_background_cache_gc_lifecycle() {
        let temp = tempdir().unwrap();
        let config = GcConfig::default();
        let gc = BackgroundCacheGc::new(temp.path().to_path_buf(), config);

        assert!(!gc.is_running());
        let handle = gc.start();
        assert!(gc.is_running());
        assert!(handle.load(Ordering::SeqCst));

        gc.stop();
        assert!(!gc.is_running());
    }

    #[test]
    fn test_scan_and_evict_expired_files() {
        let temp = tempdir().unwrap();
        let file_path = temp.path().join("expired.cache");
        std::fs::write(&file_path, b"expired payload").unwrap();

        let config = GcConfig {
            ttl: Duration::from_secs(0),
            ..GcConfig::default()
        };
        let gc = BackgroundCacheGc::new(temp.path().to_path_buf(), config);

        std::thread::sleep(Duration::from_millis(10));
        let (removed, freed) = gc.scan_and_evict();
        assert_eq!(removed, 1);
        assert!(freed > 0);
        assert!(!file_path.exists());
    }
}
