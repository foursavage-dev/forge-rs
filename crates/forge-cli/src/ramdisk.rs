#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct RamDisk {
    mount_path: PathBuf,
    is_active: bool,
}

impl RamDisk {
    pub fn create_turbo_workspace(preferred_name: &str) -> io::Result<Self> {
        #[cfg(target_os = "linux")]
        {
            let shm_path = Path::new("/dev/shm").join(format!(
                "forge_{}_{}",
                preferred_name,
                std::process::id()
            ));
            if shm_path.parent().map(|p| p.exists()).unwrap_or(false) {
                fs::create_dir_all(&shm_path)?;
                return Ok(Self {
                    mount_path: shm_path,
                    is_active: true,
                });
            }
        }

        let temp_dir = std::env::temp_dir().join(format!(
            "forge_ramdisk_{}_{}",
            preferred_name,
            std::process::id()
        ));
        fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            mount_path: temp_dir,
            is_active: true,
        })
    }

    pub fn path(&self) -> &Path {
        &self.mount_path
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn sync_artifacts_to(&self, destination: &Path) -> io::Result<usize> {
        if !self.mount_path.exists() {
            return Ok(0);
        }

        fs::create_dir_all(destination)?;
        let mut count = 0;

        for entry in fs::read_dir(&self.mount_path)? {
            let entry = entry?;
            let src = entry.path();
            let dest = destination.join(entry.file_name());

            if src.is_file() {
                fs::copy(&src, &dest)?;
                count += 1;
            } else if src.is_dir() {
                Self::copy_dir_recursive(&src, &dest)?;
                count += 1;
            }
        }

        Ok(count)
    }

    fn copy_dir_recursive(src: &Path, dest: &Path) -> io::Result<()> {
        fs::create_dir_all(dest)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let entry_src = entry.path();
            let entry_dest = dest.join(entry.file_name());
            if entry_src.is_dir() {
                Self::copy_dir_recursive(&entry_src, &entry_dest)?;
            } else {
                fs::copy(&entry_src, &entry_dest)?;
            }
        }
        Ok(())
    }
}

impl Drop for RamDisk {
    fn drop(&mut self) {
        if self.is_active && self.mount_path.exists() {
            let _ = fs::remove_dir_all(&self.mount_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramdisk_creation_and_sync() {
        let ramdisk = RamDisk::create_turbo_workspace("test_turbo").unwrap();
        assert!(ramdisk.is_active());
        assert!(ramdisk.path().exists());

        let artifact_path = ramdisk.path().join("output.bin");
        fs::write(&artifact_path, b"turbo payload").unwrap();

        let dest_dir = tempfile::tempdir().unwrap();
        let count = ramdisk.sync_artifacts_to(dest_dir.path()).unwrap();
        assert_eq!(count, 1);

        let synced_file = dest_dir.path().join("output.bin");
        assert!(synced_file.exists());
        assert_eq!(fs::read(&synced_file).unwrap(), b"turbo payload");
    }
}
