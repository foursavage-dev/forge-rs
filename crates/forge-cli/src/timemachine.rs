#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSnapshot {
    pub id: String,
    pub timestamp: u64,
    pub project_name: String,
    pub git_ref: Option<String>,
    pub artifacts: HashMap<String, String>,
    pub total_artifacts: usize,
}

pub struct TimeMachine {
    storage_dir: PathBuf,
}

impl TimeMachine {
    pub fn new(project_root: &Path) -> Self {
        let storage_dir = project_root.join(".forge").join("history");
        Self { storage_dir }
    }

    pub fn record_snapshot(
        &self,
        project_name: &str,
        artifacts: HashMap<String, String>,
    ) -> io::Result<BuildSnapshot> {
        fs::create_dir_all(&self.storage_dir)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let id = format!("snap_{:x}_{}", now, artifacts.len());
        let total_artifacts = artifacts.len();

        let snapshot = BuildSnapshot {
            id: id.clone(),
            timestamp: now,
            project_name: project_name.to_string(),
            git_ref: Self::detect_git_ref(),
            artifacts,
            total_artifacts,
        };

        let file_path = self.storage_dir.join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(file_path, content)?;

        Ok(snapshot)
    }

    pub fn list_snapshots(&self) -> io::Result<Vec<BuildSnapshot>> {
        if !self.storage_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();
        for entry in fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(snap) = serde_json::from_str::<BuildSnapshot>(&content) {
                        snapshots.push(snap);
                    }
                }
            }
        }

        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(snapshots)
    }

    pub fn rewind_to_snapshot(
        &self,
        snapshot_id: &str,
        destination_dir: &Path,
    ) -> io::Result<usize> {
        let snapshots = self.list_snapshots()?;
        let target = snapshots
            .iter()
            .find(|s| s.id == snapshot_id || s.id.starts_with(snapshot_id))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Snapshot `{}` not found in history", snapshot_id),
                )
            })?;

        fs::create_dir_all(destination_dir)?;
        let mut restored = 0;

        for (rel_path, hash) in &target.artifacts {
            let dest_file = destination_dir.join(rel_path);
            if let Some(parent) = dest_file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dest_file, format!("FORGE_RESTORED_BLOB:{}", hash))?;
            restored += 1;
        }

        Ok(restored)
    }

    fn detect_git_ref() -> Option<String> {
        std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_time_machine_record_list_and_rewind() {
        let temp = tempdir().unwrap();
        let tm = TimeMachine::new(temp.path());

        let mut artifacts = HashMap::new();
        artifacts.insert("bin/app.exe".to_string(), "hash_abc_123".to_string());
        artifacts.insert("lib/core.rlib".to_string(), "hash_def_456".to_string());

        let snapshot = tm.record_snapshot("demo_project", artifacts).unwrap();
        assert_eq!(snapshot.total_artifacts, 2);

        let list = tm.list_snapshots().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, snapshot.id);

        let restore_dir = temp.path().join("restored_target");
        let count = tm.rewind_to_snapshot(&snapshot.id, &restore_dir).unwrap();
        assert_eq!(count, 2);
        assert!(restore_dir.join("bin/app.exe").exists());
        assert!(restore_dir.join("lib/core.rlib").exists());
    }
}
