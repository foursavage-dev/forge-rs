use std::path::Path;

use crate::{BackendError, BuildMode};
use fish_core::{DEFAULT_EXCLUDED_DIRS, FingerprintUtils};

pub const EXCLUDED_DIRS: &[&str] = DEFAULT_EXCLUDED_DIRS;

pub fn hash_string(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(value.as_bytes());
}

pub fn hash_file_into(path: &Path, hasher: &mut blake3::Hasher) -> Result<(), BackendError> {
    FingerprintUtils::hash_file_into(path, hasher).map_err(|source| BackendError::Read {
        path: path.to_path_buf(),
        source,
    })
}

pub fn hash_directory(dir: &Path, hasher: &mut blake3::Hasher) -> Result<(), BackendError> {
    FingerprintUtils::hash_directory_filtered(
        dir,
        |name| EXCLUDED_DIRS.contains(&name),
        |_| true,
        hasher,
    )
    .map_err(|source| BackendError::Read {
        path: dir.to_path_buf(),
        source,
    })
}

pub fn package_input_fingerprint(
    package_dir: &Path,
    lock_file: Option<&Path>,
    toolchain: &str,
    mode: BuildMode,
) -> Result<String, BackendError> {
    let mut hasher = blake3::Hasher::new();
    hash_directory(package_dir, &mut hasher)?;
    if let Some(lock) = lock_file {
        hasher.update(b"lock:");
        hash_file_into(lock, &mut hasher)?;
    }
    hash_string(
        &mut hasher,
        &format!(
            "profile=debug;toolchain={};mode={}",
            toolchain,
            mode.as_str()
        ),
    );
    Ok(hasher.finalize().to_hex().to_string())
}

pub fn combined_fingerprint(own: &str, dep_fingerprints: &[String]) -> String {
    FingerprintUtils::combine_fingerprints(own, dep_fingerprints)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn hashes_all_files_recursively_and_stably() {
        let dir = temp_dir();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub fn f() {}").unwrap();
        let mut hasher = blake3::Hasher::new();
        hash_directory(dir.path(), &mut hasher).unwrap();
        let first = hasher.finalize().to_hex().to_string();

        let dir2 = temp_dir();
        fs::create_dir_all(dir2.path().join("src")).unwrap();
        fs::write(dir2.path().join("Cargo.toml"), "[package]").unwrap();
        fs::write(dir2.path().join("src/lib.rs"), "pub fn f() {}").unwrap();
        let mut hasher = blake3::Hasher::new();
        hash_directory(dir2.path(), &mut hasher).unwrap();
        let second = hasher.finalize().to_hex().to_string();

        assert_eq!(first, second, "identical trees must hash identically");
    }

    #[test]
    fn content_changes_invalidate_the_hash() {
        let dir = temp_dir();
        fs::write(dir.path().join("lib.rs"), "version one").unwrap();
        let a = {
            let mut hasher = blake3::Hasher::new();
            hash_directory(dir.path(), &mut hasher).unwrap();
            hasher.finalize().to_hex().to_string()
        };
        fs::write(dir.path().join("lib.rs"), "version two").unwrap();
        let b = {
            let mut hasher = blake3::Hasher::new();
            hash_directory(dir.path(), &mut hasher).unwrap();
            hasher.finalize().to_hex().to_string()
        };
        assert_ne!(a, b);
    }

    #[test]
    fn mtime_changes_do_not_invalidate_the_hash() {
        let dir = temp_dir();
        let path = dir.path().join("lib.rs");
        fs::write(&path, "stable content").unwrap();
        let a = {
            let mut hasher = blake3::Hasher::new();
            hash_directory(dir.path(), &mut hasher).unwrap();
            hasher.finalize().to_hex().to_string()
        };

        std::thread::sleep(std::time::Duration::from_millis(1100));
        fs::write(&path, "stable content").unwrap();
        let b = {
            let mut hasher = blake3::Hasher::new();
            hash_directory(dir.path(), &mut hasher).unwrap();
            hasher.finalize().to_hex().to_string()
        };
        assert_eq!(a, b);
    }

    #[test]
    fn excluded_directories_are_ignored() {
        let dir = temp_dir();
        fs::create_dir_all(dir.path().join("target").join("debug")).unwrap();
        fs::write(dir.path().join("target/debug/whatever.o"), "artifact").unwrap();
        let mut hasher = blake3::Hasher::new();
        hash_directory(dir.path(), &mut hasher).unwrap();
        let with_target = hasher.finalize().to_hex().to_string();

        let dir2 = temp_dir();
        let mut hasher = blake3::Hasher::new();
        hash_directory(dir2.path(), &mut hasher).unwrap();
        let empty = hasher.finalize().to_hex().to_string();

        assert_eq!(with_target, empty);
    }

    #[test]
    fn package_fingerprint_includes_the_lock_file() {
        let dir = temp_dir();
        fs::write(dir.path().join("Cargo.lock"), "lock v1").unwrap();
        let a = package_input_fingerprint(
            dir.path(),
            Some(&dir.path().join("Cargo.lock")),
            "t",
            BuildMode::Build,
        )
        .unwrap();
        fs::write(dir.path().join("Cargo.lock"), "lock v2").unwrap();
        let b = package_input_fingerprint(
            dir.path(),
            Some(&dir.path().join("Cargo.lock")),
            "t",
            BuildMode::Build,
        )
        .unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn combined_fingerprint_is_order_independent_and_union_sensitive() {
        let deps = vec![String::from("a"), String::from("bb"), String::from("ccc")];
        let mut reversed = deps.clone();
        reversed.reverse();
        assert_eq!(
            combined_fingerprint("own", &deps),
            combined_fingerprint("own", &reversed)
        );
        let missing_dep = vec![String::from("a"), String::from("bb")];
        assert_ne!(
            combined_fingerprint("own", &deps),
            combined_fingerprint("own", &missing_dep)
        );
        assert_ne!(
            combined_fingerprint("own1", &deps),
            combined_fingerprint("own2", &deps)
        );
    }
}
