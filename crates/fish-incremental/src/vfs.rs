use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VfsNodeState {
    Clean,
    Modified,
    Deleted,
    Created,
}

#[derive(Debug, Clone)]
pub struct VfsFileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub mtime: SystemTime,
    pub content_hash: String,
    pub state: VfsNodeState,
}

#[derive(Debug, Clone, Default)]
pub struct VfsSnapshotTree {
    pub root: PathBuf,
    pub entries: BTreeMap<PathBuf, VfsFileEntry>,
    pub dirty_paths: BTreeSet<PathBuf>,
}

impl VfsSnapshotTree {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            entries: BTreeMap::new(),
            dirty_paths: BTreeSet::new(),
        }
    }

    pub fn insert_or_update(&mut self, path: PathBuf, size: u64, mtime: SystemTime, hash: String) {
        let is_new = !self.entries.contains_key(&path);
        let is_changed = if let Some(existing) = self.entries.get(&path) {
            existing.size != size || existing.content_hash != hash
        } else {
            true
        };

        if is_changed {
            self.dirty_paths.insert(path.clone());
            let state = if is_new {
                VfsNodeState::Created
            } else {
                VfsNodeState::Modified
            };

            self.entries.insert(
                path.clone(),
                VfsFileEntry {
                    path,
                    size,
                    mtime,
                    content_hash: hash,
                    state,
                },
            );
        }
    }

    pub fn mark_deleted(&mut self, path: &Path) {
        if let Some(entry) = self.entries.get_mut(path) {
            entry.state = VfsNodeState::Deleted;
            self.dirty_paths.insert(path.to_path_buf());
        }
    }

    pub fn flush_clean(&mut self) {
        for path in &self.dirty_paths {
            if let Some(entry) = self.entries.get_mut(path) {
                entry.state = VfsNodeState::Clean;
            }
        }
        self.dirty_paths.clear();
    }

    pub fn get_dirty_paths(&self) -> Vec<PathBuf> {
        self.dirty_paths.iter().cloned().collect()
    }

    pub fn is_dirty(&self) -> bool {
        !self.dirty_paths.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_tree_mutation_and_dirty_tracking() {
        let mut vfs = VfsSnapshotTree::new(PathBuf::from("/workspace"));
        let p1 = PathBuf::from("/workspace/src/lib.rs");
        let p2 = PathBuf::from("/workspace/src/main.rs");

        vfs.insert_or_update(
            p1.clone(),
            100,
            SystemTime::now(),
            "hash_initial".to_string(),
        );
        vfs.insert_or_update(p2.clone(), 200, SystemTime::now(), "hash_main".to_string());

        assert_eq!(vfs.get_dirty_paths().len(), 2);
        vfs.flush_clean();
        assert!(!vfs.is_dirty());

        vfs.insert_or_update(
            p1.clone(),
            100,
            SystemTime::now(),
            "hash_initial".to_string(),
        );
        assert!(!vfs.is_dirty());

        vfs.insert_or_update(p1.clone(), 120, SystemTime::now(), "hash_mod".to_string());
        assert_eq!(vfs.get_dirty_paths(), vec![p1]);
    }
}
