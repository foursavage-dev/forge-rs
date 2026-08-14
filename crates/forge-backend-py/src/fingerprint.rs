use blake3::Hasher;
use std::fs;
use std::io;
use std::path::Path;

pub fn compute_py_fingerprint(root: &Path, source_dirs: &[String]) -> Result<String, io::Error> {
    let mut hasher = Hasher::new();

    for config_name in &[
        "pyproject.toml",
        "setup.py",
        "setup.cfg",
        "requirements.txt",
        "uv.lock",
        "poetry.lock",
        "Pipfile.lock",
        "forge.py.json",
        "ruff.toml",
        ".ruff.toml",
        "mypy.ini",
        "pytest.ini",
    ] {
        let file_path = root.join(config_name);
        if file_path.is_file() {
            hasher.update(config_name.as_bytes());
            let bytes = fs::read(&file_path)?;
            hasher.update(&bytes);
        }
    }

    let mut dirs_to_scan = Vec::new();
    if source_dirs.is_empty() {
        dirs_to_scan.push(root.to_path_buf());
    } else {
        for dir in source_dirs {
            let full_dir = root.join(dir);
            if full_dir.exists() {
                dirs_to_scan.push(full_dir);
            }
        }
        if dirs_to_scan.is_empty() {
            dirs_to_scan.push(root.to_path_buf());
        }
    }

    for dir in dirs_to_scan {
        hash_directory_recursive(&dir, &mut hasher)?;
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn hash_directory_recursive(dir: &Path, hasher: &mut Hasher) -> Result<(), io::Error> {
    if !dir.is_dir() {
        return Ok(());
    }

    let mut entries = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if name == "__pycache__"
            || name == ".venv"
            || name == "venv"
            || name == ".pytest_cache"
            || name == ".mypy_cache"
            || name == ".ruff_cache"
            || name == "dist"
            || name == "build"
            || name == ".git"
            || name == ".forge"
            || name.ends_with(".egg-info")
            || name.starts_with('.')
        {
            continue;
        }

        if path.is_dir() {
            hash_directory_recursive(&path, hasher)?;
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if matches!(
                    ext,
                    "py" | "pyi" | "toml" | "json" | "yaml" | "yml" | "sql" | "txt"
                ) {
                    hasher.update(name.as_bytes());
                    let bytes = fs::read(&path)?;
                    hasher.update(&bytes);
                }
            }
        }
    }

    Ok(())
}
