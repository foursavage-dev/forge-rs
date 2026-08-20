#![forbid(unsafe_code)]

use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReflinkMode {
    Reflink,
    Hardlink,
    Copy,
}

pub fn reflink_or_copy(src: &Path, dst: &Path) -> io::Result<ReflinkMode> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }

    if dst.exists() {
        let _ = fs::remove_file(dst);
    }

    if let Ok(()) = fs::hard_link(src, dst) {
        return Ok(ReflinkMode::Hardlink);
    }

    fs::copy(src, dst)?;
    Ok(ReflinkMode::Copy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_reflink_or_copy_basic() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("source.dat");
        let dst = temp.path().join("target.dat");

        fs::write(&src, b"fast block data").unwrap();

        let mode = reflink_or_copy(&src, &dst).unwrap();
        assert!(
            mode == ReflinkMode::Hardlink
                || mode == ReflinkMode::Copy
                || mode == ReflinkMode::Reflink
        );
        assert_eq!(fs::read(&dst).unwrap(), b"fast block data");
    }

    #[test]
    fn test_reflink_or_copy_nested_destination() {
        let temp = tempdir().unwrap();
        let src = temp.path().join("source.dat");
        let dst = temp.path().join("nested/sub/target.dat");

        fs::write(&src, b"nested block payload").unwrap();

        let mode = reflink_or_copy(&src, &dst).unwrap();
        assert!(dst.exists());
        assert_eq!(fs::read(&dst).unwrap(), b"nested block payload");
        assert!(
            mode == ReflinkMode::Hardlink
                || mode == ReflinkMode::Copy
                || mode == ReflinkMode::Reflink
        );
    }
}
