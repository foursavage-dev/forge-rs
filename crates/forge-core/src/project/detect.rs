//! Locating a Cargo project on disk.
//!
//! Like Cargo itself, Forge searches upward from a starting directory for a
//! `Cargo.toml` manifest.

use std::path::{Path, PathBuf};

/// File name of Cargo's manifest.
pub const MANIFEST_FILE: &str = "Cargo.toml";

/// Return the first directory at or above `start` that contains a `Cargo.toml`
/// file, or `None` if no such directory exists.
pub fn find_manifest_dir(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join(MANIFEST_FILE).is_file())
        .map(Path::to_path_buf)
}

/// Return the path of the `Cargo.toml` file governing `start`, or `None` if
/// no Cargo project contains `start`.
pub fn find_manifest(start: &Path) -> Option<PathBuf> {
    find_manifest_dir(start).map(|dir| dir.join(MANIFEST_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn finds_manifest_in_starting_dir() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        fs::write(tmp.path().join(MANIFEST_FILE), "[package]\n").expect("write manifest");

        let start = tmp.path().join("src");
        fs::create_dir_all(&start).expect("create nested dir");

        assert_eq!(find_manifest(&start), Some(tmp.path().join(MANIFEST_FILE)));
        assert_eq!(find_manifest_dir(&start), Some(PathBuf::from(tmp.path())));
    }

    #[test]
    fn finds_manifest_walking_up() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        fs::write(tmp.path().join(MANIFEST_FILE), "[package]\n").expect("write manifest");

        let start = tmp.path().join("a").join("b").join("c");
        fs::create_dir_all(&start).expect("create nested dirs");

        assert_eq!(find_manifest(&start), Some(tmp.path().join(MANIFEST_FILE)));
    }

    #[test]
    fn returns_none_without_manifest() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        assert_eq!(find_manifest(tmp.path()), None);
        assert_eq!(find_manifest_dir(tmp.path()), None);
    }

    #[test]
    fn ignores_manifest_in_sibling_directory() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        fs::create_dir_all(tmp.path().join("other")).expect("create sibling");
        fs::write(tmp.path().join("other").join(MANIFEST_FILE), "[package]\n")
            .expect("write manifest");

        let start = tmp.path().join("work");
        fs::create_dir_all(&start).expect("create work dir");

        assert_eq!(find_manifest(&start), None);
    }
}
