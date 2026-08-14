#![forbid(unsafe_code)]

use crate::SwiftBackendError;
use crate::config::SwiftPlatform;
use std::path::Path;
use std::fs;

pub fn compute_swift_fingerprint(
    project_dir: &Path,
    swift_version: &str,
    platform: &SwiftPlatform,
) -> Result<String, SwiftBackendError> {
    let mut hasher = blake3::Hasher::new();

    // Include toolchain version
    hasher.update(swift_version.as_bytes());

    // Include platform
    hasher.update(platform.as_str().as_bytes());

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

fn collect_source_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, SwiftBackendError> {
    let mut source_files = Vec::new();
    let extensions = [".swift", ".m", ".mm", ".h", ".hpp", ".c", ".cpp"];
    
    let source_dirs = [
        project_dir.join("Sources"),
        project_dir.join("Tests"),
        project_dir.join("src"),
        project_dir.join("test"),
        project_dir.join("Classes"),
        project_dir.join("include"),
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

fn collect_config_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, SwiftBackendError> {
    let mut config_files = Vec::new();

    // Package.swift
    let package_swift = project_dir.join("Package.swift");
    if package_swift.exists() {
        config_files.push(package_swift);
    }

    // Package.resolved
    let package_resolved = project_dir.join("Package.resolved");
    if package_resolved.exists() {
        config_files.push(package_resolved);
    }

    // .xcodeproj files
    if let Ok(entries) = fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("xcodeproj") {
                config_files.push(path.clone());
                // Also add contents of .xcodeproj
                if let Ok(xcode_entries) = fs::read_dir(&path) {
                    for xcode_entry in xcode_entries.flatten() {
                        let xcode_path = xcode_entry.path();
                        if xcode_path.is_file() {
                            config_files.push(xcode_path);
                        }
                    }
                }
            }
        }
    }

    Ok(config_files)
}

fn collect_dependency_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, SwiftBackendError> {
    let mut dependency_files = Vec::new();

    // Podfile (CocoaPods)
    let podfile = project_dir.join("Podfile");
    if podfile.exists() {
        dependency_files.push(podfile);
    }

    // Podfile.lock
    let podfile_lock = project_dir.join("Podfile.lock");
    if podfile_lock.exists() {
        dependency_files.push(podfile_lock);
    }

    // Cartfile (Carthage)
    let cartfile = project_dir.join("Cartfile");
    if cartfile.exists() {
        dependency_files.push(cartfile);
    }

    // Cartfile.resolved
    let cartfile_resolved = project_dir.join("Cartfile.resolved");
    if cartfile_resolved.exists() {
        dependency_files.push(cartfile_resolved);
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
    fn test_swift_fingerprint() {
        let temp = tempdir().unwrap();
        let sources_dir = temp.path().join("Sources");
        fs::create_dir_all(&sources_dir).unwrap();

        let swift_file = sources_dir.join("main.swift");
        fs::write(&swift_file, "print(\"Hello, World!\")").unwrap();

        let package_file = temp.path().join("Package.swift");
        fs::write(&package_file, "// swift-tools-version: 5.9").unwrap();

        let fingerprint = compute_swift_fingerprint(
            temp.path(),
            "5.9.0",
            &SwiftPlatform::MacOS,
        ).unwrap();

        assert!(!fingerprint.is_empty());
        assert_eq!(fingerprint.len(), 64); // blake3 hash length
    }

    #[test]
    fn test_fingerprint_changes_with_source() {
        let temp = tempdir().unwrap();
        let sources_dir = temp.path().join("Sources");
        fs::create_dir_all(&sources_dir).unwrap();

        let swift_file = sources_dir.join("main.swift");
        fs::write(&swift_file, "print(\"Hello, World!\")").unwrap();

        let fp1 = compute_swift_fingerprint(
            temp.path(),
            "5.9.0",
            &SwiftPlatform::MacOS,
        ).unwrap();

        fs::write(&swift_file, "print(\"Goodbye, World!\")").unwrap();

        let fp2 = compute_swift_fingerprint(
            temp.path(),
            "5.9.0",
            &SwiftPlatform::MacOS,
        ).unwrap();

        assert_ne!(fp1, fp2);
    }
}
