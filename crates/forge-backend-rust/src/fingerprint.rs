//! Input-fingerprinting for the Rust backend.
//!
//! A package's fingerprint is a content hash of:
//!
//! 1. every regular file under the package directory (excluding `target`,
//!    `.git`, and `.forge`), walked deterministically;
//! 2. the workspace `Cargo.lock`, when present (so dependency bumps
//!    invalidate fingerprints);
//! 3. the toolchain and profile configuration.
//!
//! The package-level fingerprint is combined with the fingerprints of all
//! direct dependencies (sorted to be order-independent) so a change anywhere
//! in the dependency cone invalidates every affected task. Only file
//! *content* is hashed; mtimes are deliberately ignored for robustness.

use std::fs;
use std::io::Read;
use std::path::Path;

use crate::{BackendError, BuildMode};

/// Directories that never contribute to a package fingerprint.
pub const EXCLUDED_DIRS: &[&str] = &["target", ".git", ".forge"];

/// Hash a string into `hasher` (NOT length-prefixed; callers control
/// framing at the next level).
pub fn hash_string(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(value.as_bytes());
}

/// Hash the content of a single file into `hasher`.
pub fn hash_file_into(path: &Path, hasher: &mut blake3::Hasher) -> Result<(), BackendError> {
    let mut file = fs::File::open(path).map_err(|source| BackendError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|source| BackendError::Read {
                path: path.to_path_buf(),
                source,
            })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(())
}

/// Hash every regular file under `dir` (recursively) into `hasher`.
///
/// Entries are visited in sorted order with their *relative* path hashed
/// before the content, so the hash is stable across machines and unaffected
/// by where the workspace lives. Symlinks and excluded directories are
/// skipped.
pub fn hash_directory(dir: &Path, hasher: &mut blake3::Hasher) -> Result<(), BackendError> {
    fn walk(hasher: &mut blake3::Hasher, dir: &Path, base: &str) -> Result<(), BackendError> {
        let mut entries = fs::read_dir(dir)
            .map_err(|source| BackendError::Read {
                path: dir.to_path_buf(),
                source,
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|source| BackendError::Read {
                path: dir.to_path_buf(),
                source,
            })?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name();
            if EXCLUDED_DIRS.iter().any(|excluded| name == *excluded) {
                continue;
            }
            let relative = if base.is_empty() {
                name.to_string_lossy().into_owned()
            } else {
                format!("{base}/{}", name.to_string_lossy())
            };
            let path = entry.path();
            let file_type = entry.file_type().map_err(|source| BackendError::Read {
                path: path.clone(),
                source,
            })?;
            hasher.update(relative.as_bytes());
            if file_type.is_dir() {
                walk(hasher, &path, &relative)?;
            } else if file_type.is_file() {
                hash_file_into(&path, hasher)?;
            }
            // Symlinks and special files are hashed as metadata only
            // (their name already went into the hash above).
        }
        Ok(())
    }
    walk(hasher, dir, "")
}

/// Fingerprint of a package's own inputs, independent of its dependencies.
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
            "profile={};toolchain={};mode={}",
            crate::PROFILE,
            toolchain,
            mode.as_str()
        ),
    );
    Ok(hasher.finalize().to_hex().to_string())
}

/// Combine a package's own fingerprint with its dependencies' fingerprints.
///
/// Dependencies are sorted so the combination is order-independent, and each
/// fingerprint is length-framed to avoid ambiguous concatenations.
pub fn combined_fingerprint(own: &str, dep_fingerprints: &[String]) -> String {
    let mut deps: Vec<&String> = dep_fingerprints.iter().collect();
    deps.sort();
    let mut hasher = blake3::Hasher::new();
    hasher.update(format!("own[{}]:", own.len()).as_bytes());
    hasher.update(own.as_bytes());
    for dep in deps {
        hasher.update(format!("dep[{}]:", dep.len()).as_bytes());
        hasher.update(dep.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
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
        // Rewrite identical content after a pause: same bytes, new mtime.
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
