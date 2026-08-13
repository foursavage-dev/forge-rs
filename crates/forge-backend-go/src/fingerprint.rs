use std::fs;
use std::path::Path;

pub fn compute_go_fingerprint(
    project_dir: &Path,
    toolchain_version: &str,
    tags: &[String],
) -> Result<String, std::io::Error> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(toolchain_version.as_bytes());
    hasher.update(b"\ntags:\n");
    for tag in tags {
        hasher.update(tag.as_bytes());
        hasher.update(b"\n");
    }

    let go_mod = project_dir.join("go.mod");
    if go_mod.exists() {
        if let Ok(content) = fs::read(&go_mod) {
            hasher.update(&content);
        }
    }

    let go_sum = project_dir.join("go.sum");
    if go_sum.exists() {
        if let Ok(content) = fs::read(&go_sum) {
            hasher.update(&content);
        }
    }

    hash_dir_go_files(project_dir, &mut hasher)?;

    Ok(hasher.finalize().to_hex().to_string())
}

fn hash_dir_go_files(dir: &Path, hasher: &mut blake3::Hasher) -> Result<(), std::io::Error> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "go" {
                    entries.push(path);
                }
            }
        } else if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') && name != "vendor" {
                hash_dir_go_files(&path, hasher)?;
            }
        }
    }

    entries.sort();
    for path in entries {
        if let Ok(content) = fs::read(&path) {
            hasher.update(path.to_string_lossy().as_bytes());
            hasher.update(&content);
        }
    }

    Ok(())
}
