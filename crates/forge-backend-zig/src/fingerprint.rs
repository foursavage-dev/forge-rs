#![forbid(unsafe_code)]

use crate::ZigBackendError;
use crate::config::ZigTarget;
use std::path::Path;
use std::fs;

pub fn compute_zig_fingerprint(
    project_dir: &Path,
    zig_version: &str,
    target: &ZigTarget,
) -> Result<String, ZigBackendError> {
    let mut hasher = blake3::Hasher::new();

    // Include toolchain version
    hasher.update(zig_version.as_bytes());

    // Include target
    hasher.update(target.as_str().as_bytes());

    // Include source files
    let source_files = collect_source_files(project_dir)?;
    for file in &source_files {
        if let Ok(content) = fs::read(file) {
            hasher.update(&content);
        }
    }

    // Include project configuration
    let config_files = collect_config_files(project_dir)?;
    for file in &config_files {
        if let Ok(content) = fs::read(file) {
            hasher.update(&content);
        }
    }

    // Include dependency files
    let dependency_files = collect_dependency_files(project_dir)?;
    for file in &dependency_files {
        if let Ok(content) = fs::read(file) {
            hasher.update(&content);
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn collect_source_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, ZigBackendError> {
    let mut source_files = Vec::new();
    let extensions = ["zig", "zon"];
    
    let source_dirs = [
        project_dir.join("src"),
        project_dir.join("lib"),
        project_dir.join("test"),
    ];

    for source_dir in &source_dirs {
        if source_dir.exists() {
            collect_files_recursive(source_dir, &extensions, &mut source_files);
        }
    }

    // Also check subdirectories for source files
    if let Ok(entries) = fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_recursive(&path, &extensions, &mut source_files);
            }
        }
    }

    Ok(source_files)
}

fn collect_config_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, ZigBackendError> {
    let mut config_files = Vec::new();

    // build.zig
    let build_zig = project_dir.join("build.zig");
    if build_zig.exists() {
        config_files.push(build_zig);
    }

    // build.zig.zon (if exists)
    let build_zig_zon = project_dir.join("build.zig.zon");
    if build_zig_zon.exists() {
        config_files.push(build_zig_zon);
    }

    Ok(config_files)
}

fn collect_dependency_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, ZigBackendError> {
    let mut dependency_files = Vec::new();

    // zig.zon (dependency manifest)
    let zig_zon = project_dir.join("zig.zon");
    if zig_zon.exists() {
        dependency_files.push(zig_zon);
    }

    // Check zig-cache directory
    let zig_cache = project_dir.join("zig-cache");
    if zig_cache.exists() {
        if let Ok(entries) = fs::read_dir(&zig_cache) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    dependency_files.push(path);
                }
            }
        }
    }

    // Check zigs-out directory
    let zigs_out = project_dir.join("zig-out");
    if zigs_out.exists() {
        if let Ok(entries) = fs::read_dir(&zigs_out) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    dependency_files.push(path);
                }
            }
        }
    }

    Ok(dependency_files)
}

fn collect_files_recursive(dir: &Path, extensions: &[&str], collected: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_recursive(&path, extensions, collected);
            } else if let Some(ext) = path.extension() {
                if extensions.iter().any(|e| *e == ext) {
                    collected.push(path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_zig_fingerprint() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let zig_file = src_dir.join("main.zig");
        fs::write(&zig_file, "const std = @import(\"std\"); pub fn main() !void { std.debug.print(\"Hello\\n\"); }").unwrap();

        let build_file = temp.path().join("build.zig");
        fs::write(&build_file, "const std = @import(\"std\");").unwrap();

        let fingerprint = compute_zig_fingerprint(
            temp.path(),
            "0.11.0",
            &ZigTarget::Native,
        ).unwrap();

        assert!(!fingerprint.is_empty());
        assert_eq!(fingerprint.len(), 64); // blake3 hash length
    }

    #[test]
    fn test_fingerprint_changes_with_source() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let zig_file = src_dir.join("main.zig");
        fs::write(&zig_file, "const std = @import(\"std\"); pub fn main() !void { std.debug.print(\"Hello\\n\"); }").unwrap();

        let fp1 = compute_zig_fingerprint(
            temp.path(),
            "0.11.0",
            &ZigTarget::Native,
        ).unwrap();

        fs::write(&zig_file, "const std = @import(\"std\"); pub fn main() !void { std.debug.print(\"Goodbye\\n\"); }").unwrap();

        let fp2 = compute_zig_fingerprint(
            temp.path(),
            "0.11.0",
            &ZigTarget::Native,
        ).unwrap();

        assert_ne!(fp1, fp2);
    }
}
