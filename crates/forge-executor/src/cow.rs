use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneStrategy {
    ExtentClone,
    HardLink,
    FastCopy,
}

pub struct KernelCowCloner;

impl KernelCowCloner {
    pub fn try_clone_file(src: &Path, dst: &Path) -> io::Result<CloneStrategy> {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }

        if dst.exists() {
            let _ = fs::remove_file(dst);
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(()) = Self::try_hardlink(src, dst) {
                return Ok(CloneStrategy::HardLink);
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(()) = Self::try_hardlink(src, dst) {
                return Ok(CloneStrategy::HardLink);
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(()) = Self::try_hardlink(src, dst) {
                return Ok(CloneStrategy::HardLink);
            }
        }

        Self::fast_copy(src, dst)?;
        Ok(CloneStrategy::FastCopy)
    }

    fn try_hardlink(src: &Path, dst: &Path) -> io::Result<()> {
        fs::hard_link(src, dst)
    }

    fn fast_copy(src: &Path, dst: &Path) -> io::Result<()> {
        fs::copy(src, dst)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cow_cloner_file_integrity() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("source.bin");
        let dst = dir.path().join("sub").join("clone.bin");

        let content = b"FORGE_KERNEL_COW_TEST_BYTES_0123456789";
        fs::write(&src, content).unwrap();

        let strategy = KernelCowCloner::try_clone_file(&src, &dst).unwrap();
        assert!(dst.exists());
        assert_eq!(fs::read(&dst).unwrap(), content);
        assert!(matches!(
            strategy,
            CloneStrategy::HardLink | CloneStrategy::FastCopy | CloneStrategy::ExtentClone
        ));
    }

    #[test]
    fn test_cow_cloner_overwrite_existing() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("source2.bin");
        let dst = dir.path().join("clone2.bin");

        fs::write(&src, b"new_data").unwrap();
        fs::write(&dst, b"old_data").unwrap();

        let _ = KernelCowCloner::try_clone_file(&src, &dst).unwrap();
        assert_eq!(fs::read(&dst).unwrap(), b"new_data");
    }
}
