#![forbid(unsafe_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn other(error: impl std::error::Error + Send + Sync + 'static) -> ArtifactError {
    ArtifactError(io::Error::other(error))
}

/// Builds a content-addressed blob: the given paths (relative to `root`,
/// files or directories) are packed into a tar stream and compressed with
/// zstd. The blob is the payload stored in both the local object store and
/// the remote cache; its identity is `blob_hash(blob)`.
pub fn pack_artifacts(root: &Path, paths: &[PathBuf]) -> Result<Vec<u8>, ArtifactError> {
    let mut files: Vec<PathBuf> = Vec::new();
    for path in paths {
        let full = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        let Ok(metadata) = fs::metadata(&full) else {
            continue;
        };
        if metadata.is_dir() {
            collect_files(&full, &mut files)?;
        } else if metadata.is_file() {
            files.push(full);
        }
    }

    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        for full in files {
            let relative = full.strip_prefix(root).unwrap_or(&full).to_path_buf();
            let metadata = fs::metadata(&full)?;
            let mut file = fs::File::open(&full)?;
            let mut header = tar::Header::new_gnu();
            header.set_metadata(&metadata);
            header.set_size(metadata.len());
            header.set_cksum();
            builder.append_data(&mut header, &relative, &mut file)?;
        }
        builder.finish()?;
    }
    compress(&tar_bytes)
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), ArtifactError> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

/// Packs an entire source tree (everything under `root` except entries whose
/// file name matches one of `excludes`) into a compressed tar blob. Used to
/// ship source context to remote workers.
pub fn pack_tree(root: &Path, excludes: &[&str]) -> Result<Vec<u8>, ArtifactError> {
    let mut files = Vec::new();
    collect_tree(root, excludes, &mut files)?;
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        for full in files {
            let relative = full.strip_prefix(root).unwrap_or(&full).to_path_buf();
            let metadata = fs::metadata(&full)?;
            let mut file = fs::File::open(&full)?;
            let mut header = tar::Header::new_gnu();
            header.set_metadata(&metadata);
            header.set_size(metadata.len());
            header.set_cksum();
            builder.append_data(&mut header, &relative, &mut file)?;
        }
        builder.finish()?;
    }
    compress(&tar_bytes)
}

fn collect_tree(
    dir: &Path,
    excludes: &[&str],
    out: &mut Vec<PathBuf>,
) -> Result<(), ArtifactError> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| excludes.contains(&name))
        {
            continue;
        }
        if path.is_dir() {
            collect_tree(&path, excludes, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

/// Decompresses and extracts a tar.zst blob into `root`, skipping entries
/// that would escape the root (path traversal guard).
pub fn unpack_artifacts(blob: &[u8], root: &Path) -> Result<(), ArtifactError> {
    fs::create_dir_all(root)?;
    let decoded = decompress(blob)?;
    let mut archive = tar::Archive::new(decoded.as_slice());
    for entry in archive.entries().map_err(other)? {
        let mut entry = entry.map_err(other)?;
        let path = entry.path().map_err(other)?.into_owned();
        if path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            continue;
        }
        entry.unpack_in(root).map_err(other)?;
    }
    Ok(())
}

pub fn compress(bytes: &[u8]) -> Result<Vec<u8>, ArtifactError> {
    let level = 3;
    Ok(zstd::stream::encode_all(bytes, level)?)
}

pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>, ArtifactError> {
    let mut out = Vec::new();
    zstd::stream::copy_decode(bytes, &mut out)?;
    Ok(out)
}

/// Content address of a blob: blake3 of the (already compressed) bytes.
pub fn blob_hash(blob: &[u8]) -> String {
    blake3::hash(blob).to_hex().to_string()
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ArtifactError(#[from] io::Error);

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn pack_unpack_roundtrip_restores_content() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("out");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("app.exe"), b"fake binary").unwrap();
        fs::write(sub.join("lib.so"), b"shared lib").unwrap();

        let blob = pack_artifacts(dir.path(), std::slice::from_ref(&sub)).unwrap();
        assert!(!blob.is_empty());

        let dest = dir.path().join("restored");
        unpack_artifacts(&blob, &dest).unwrap();
        assert_eq!(fs::read(dest.join("out/app.exe")).unwrap(), b"fake binary");
        assert_eq!(fs::read(dest.join("out/lib.so")).unwrap(), b"shared lib");
    }

    #[test]
    fn pack_skips_missing_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("real.bin"), b"x").unwrap();
        let blob = pack_artifacts(dir.path(), &[PathBuf::from("real.bin"), PathBuf::from("missing.bin")])
            .unwrap();
        let dest = dir.path().join("r");
        unpack_artifacts(&blob, &dest).unwrap();
        assert!(dest.join("real.bin").exists());
        assert!(!dest.join("missing.bin").exists());
    }

    #[test]
    fn unpack_rejects_path_traversal_entries() {
        fn raw_tar_entry(name: &str, data: &[u8]) -> Vec<u8> {
            let mut header = [0u8; 512];
            header[..name.len().min(100)].copy_from_slice(&name.as_bytes()[..name.len().min(100)]);
            header[100..108].copy_from_slice(b"0000644\0");
            header[124..136].copy_from_slice(format!("{:011o}\0", data.len()).as_bytes());
            header[156] = b'0';
            header[257..262].copy_from_slice(b"ustar");
            for byte in &mut header[148..156] {
                *byte = b' ';
            }
            let sum: u32 = header.iter().map(|&b| b as u32).sum();
            header[148..156].copy_from_slice(format!("{:06o}\0 ", sum).as_bytes());
            let mut out = header.to_vec();
            out.extend_from_slice(data);
            let pad = (512 - data.len() % 512) % 512;
            out.extend(std::iter::repeat_n(0u8, pad));
            out.extend([0u8; 1024]);
            out
        }

        let dir = tempdir().unwrap();
        let tar_bytes = raw_tar_entry("../../escape.txt", b"evil");
        let blob = compress(&tar_bytes).unwrap();
        let dest = dir.path().join("safe");
        let result = unpack_artifacts(&blob, &dest);
        assert!(
            result.is_ok(),
            "unpack should skip the traversal entry: {:?}",
            result
        );
        assert!(!dir.path().join("escape.txt").exists());
        assert!(!dest.join("escape.txt").exists());
    }

    #[test]
    fn blob_hash_is_content_addressed_and_stable() {
        let a = blob_hash(b"hello");
        let b = blob_hash(b"hello");
        let c = blob_hash(b"hello!");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn pack_tree_excludes_common_dirs() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::create_dir_all(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("target/app.exe"), b"artifact").unwrap();
        fs::write(dir.path().join(".gitignore"), "").unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/config"), "x").unwrap();

        let blob = pack_tree(dir.path(), &[".git", "target", "node_modules"]).unwrap();
        let dest = dir.path().join("restored");
        unpack_artifacts(&blob, &dest).unwrap();
        assert!(dest.join("src/main.rs").exists());
        assert!(dest.join(".gitignore").exists());
        assert!(!dest.join("target/app.exe").exists());
        assert!(!dest.join(".git/config").exists());
    }
}
