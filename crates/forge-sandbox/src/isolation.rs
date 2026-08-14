use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FsPolicy {
    #[default]
    InPlace,
    IsolatedTemp,
}

#[derive(Debug)]
pub struct SandboxWorkspace {
    root: PathBuf,
    is_temp: bool,
}

impl SandboxWorkspace {
    pub fn in_place(path: impl Into<PathBuf>) -> Self {
        Self {
            root: path.into(),
            is_temp: false,
        }
    }

    pub fn isolated(base_dir: &Path) -> Result<Self, io::Error> {
        let temp_dir = tempfile::Builder::new()
            .prefix("forge_sb_")
            .tempdir_in(base_dir)?;
        let path = temp_dir.keep();
        Ok(Self {
            root: path,
            is_temp: true,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn copy_file(&self, src: &Path, rel_dest: &Path) -> Result<PathBuf, io::Error> {
        let dest = self.root.join(rel_dest);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, &dest)?;
        Ok(dest)
    }

    pub fn cleanup(self) -> Result<(), io::Error> {
        if self.is_temp && self.root.exists() {
            fs::remove_dir_all(&self.root)?;
        }
        Ok(())
    }
}

impl Drop for SandboxWorkspace {
    fn drop(&mut self) {
        if self.is_temp && self.root.exists() {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
