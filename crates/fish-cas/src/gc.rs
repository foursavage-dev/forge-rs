use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CasGcConfig {
    pub max_storage_bytes: u64,
    pub prune_unreferenced: bool,
    pub run_interval: Duration,
}

impl Default for CasGcConfig {
    fn default() -> Self {
        Self {
            max_storage_bytes: 50 * 1024 * 1024 * 1024,
            prune_unreferenced: true,
            run_interval: Duration::from_secs(600),
        }
    }
}

pub struct CasGarbageCollector {
    storage_root: PathBuf,
    config: CasGcConfig,
    active: Arc<AtomicBool>,
}

impl CasGarbageCollector {
    pub fn new(storage_root: PathBuf, config: CasGcConfig) -> Self {
        Self {
            storage_root,
            config,
            active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) {
        self.active.store(true, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.active.store(false, Ordering::SeqCst);
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub fn prune_unreferenced_blobs(&self, referenced_hashes: &HashSet<String>) -> (usize, u64) {
        let mut pruned = 0;
        let mut freed = 0;

        let objects_dir = self.storage_root.join("objects");
        if let Ok(entries) = std::fs::read_dir(objects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if !referenced_hashes.contains(file_name) {
                        if let Ok(meta) = path.metadata() {
                            let size = meta.len();
                            if std::fs::remove_file(&path).is_ok() {
                                pruned += 1;
                                freed += size;
                            }
                        }
                    }
                }
            }
        }

        (pruned, freed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cas_gc_pruning() {
        let temp = tempdir().unwrap();
        let objects_dir = temp.path().join("objects");
        std::fs::create_dir_all(&objects_dir).unwrap();

        let blob_a = objects_dir.join("hash_a");
        let blob_b = objects_dir.join("hash_b");
        std::fs::write(&blob_a, b"content_a").unwrap();
        std::fs::write(&blob_b, b"content_b").unwrap();

        let mut referenced = HashSet::new();
        referenced.insert("hash_a".to_string());

        let gc = CasGarbageCollector::new(temp.path().to_path_buf(), CasGcConfig::default());
        let (pruned, freed) = gc.prune_unreferenced_blobs(&referenced);

        assert_eq!(pruned, 1);
        assert!(freed > 0);
        assert!(blob_a.exists());
        assert!(!blob_b.exists());
    }
}
