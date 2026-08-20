use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotBackend {
    OverlayFs,
    BtrfsSubvolume,
    DirectoryCopy,
}

#[derive(Debug, Clone)]
pub struct SandboxSnapshot {
    pub id: String,
    pub upper_dir: PathBuf,
    pub work_dir: PathBuf,
    pub merged_dir: PathBuf,
    pub backend: SnapshotBackend,
    pub created_at: SystemTime,
}

pub struct SnapshotManager {
    base_root: PathBuf,
}

impl SnapshotManager {
    pub fn new(base_root: PathBuf) -> Self {
        Self { base_root }
    }

    pub fn create_snapshot(
        &self,
        snapshot_id: &str,
        _lower_dir: &Path,
    ) -> Result<SandboxSnapshot, std::io::Error> {
        let snap_dir = self.base_root.join(snapshot_id);
        let upper_dir = snap_dir.join("upper");
        let work_dir = snap_dir.join("work");
        let merged_dir = snap_dir.join("merged");

        std::fs::create_dir_all(&upper_dir)?;
        std::fs::create_dir_all(&work_dir)?;
        std::fs::create_dir_all(&merged_dir)?;

        let backend = if cfg!(target_os = "linux") {
            SnapshotBackend::OverlayFs
        } else {
            SnapshotBackend::DirectoryCopy
        };

        Ok(SandboxSnapshot {
            id: snapshot_id.to_string(),
            upper_dir,
            work_dir,
            merged_dir,
            backend,
            created_at: SystemTime::now(),
        })
    }

    pub fn restore_snapshot(&self, snapshot: &SandboxSnapshot) -> Result<(), std::io::Error> {
        if snapshot.upper_dir.exists() {
            std::fs::remove_dir_all(&snapshot.upper_dir)?;
            std::fs::create_dir_all(&snapshot.upper_dir)?;
        }
        if snapshot.work_dir.exists() {
            std::fs::remove_dir_all(&snapshot.work_dir)?;
            std::fs::create_dir_all(&snapshot.work_dir)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_snapshot_creation_and_fast_restore() {
        let temp = tempdir().unwrap();
        let mgr = SnapshotManager::new(temp.path().to_path_buf());
        let lower = temp.path().join("source_lower");
        std::fs::create_dir_all(&lower).unwrap();

        let snap = mgr.create_snapshot("snap_01", &lower).unwrap();
        assert!(snap.upper_dir.exists());

        let dirty_file = snap.upper_dir.join("temp.txt");
        std::fs::write(&dirty_file, b"dirty").unwrap();
        assert!(dirty_file.exists());

        mgr.restore_snapshot(&snap).unwrap();
        assert!(!dirty_file.exists());
    }
}
