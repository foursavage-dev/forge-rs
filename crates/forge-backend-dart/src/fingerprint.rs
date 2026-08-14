#![forbid(unsafe_code)]

use crate::DartBackendError;
use crate::config::DartProjectType;
use std::path::Path;
use std::fs;

pub fn compute_dart_fingerprint(
    project_dir: &Path,
    dart_version: &str,
    flutter_version: &Option<String>,
    project_type: &DartProjectType,
) -> Result<String, DartBackendError> {
    let mut hasher = blake3::Hasher::new();

    // Include toolchain versions
    hasher.update(dart_version.as_bytes());
    if let Some(flutter_ver) = flutter_version {
        hasher.update(flutter_ver.as_bytes());
    }

    // Include project type
    match project_type {
        DartProjectType::Dart => {
            hasher.update(b"dart");
        }
        DartProjectType::Flutter => {
            hasher.update(b"flutter");
        }
    }

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

fn collect_source_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, DartBackendError> {
    let mut source_files = Vec::new();
    let extensions = [".dart", ".yaml", ".json"];
    
    let source_dirs = [
        project_dir.join("lib"),
        project_dir.join("bin"),
        project_dir.join("test"),
        project_dir.join("web"),
        project_dir.join("android"),
        project_dir.join("ios"),
        project_dir.join("windows"),
        project_dir.join("macos"),
        project_dir.join("linux"),
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

fn collect_config_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, DartBackendError> {
    let mut config_files = Vec::new();

    // pubspec.yaml
    let pubspec = project_dir.join("pubspec.yaml");
    if pubspec.exists() {
        config_files.push(pubspec);
    }

    // analysis_options.yaml
    let analysis_options = project_dir.join("analysis_options.yaml");
    if analysis_options.exists() {
        config_files.push(analysis_options);
    }

    // pubspec.lock
    let pubspec_lock = project_dir.join("pubspec.lock");
    if pubspec_lock.exists() {
        config_files.push(pubspec_lock);
    }

    // .metadata
    let metadata = project_dir.join(".metadata");
    if metadata.exists() {
        config_files.push(metadata);
    }

    Ok(config_files)
}

fn collect_dependency_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, DartBackendError> {
    let mut dependency_files = Vec::new();

    // .packages file (older Dart versions)
    let packages = project_dir.join(".packages");
    if packages.exists() {
        dependency_files.push(packages);
    }

    // package_config.json (newer Dart versions)
    let package_config = project_dir.join(".dart_tool").join("package_config.json");
    if package_config.exists() {
        dependency_files.push(package_config);
    }

    // Flutter-specific files
    let flutter_plugins = project_dir.join(".flutter-plugins");
    if flutter_plugins.exists() {
        dependency_files.push(flutter_plugins);
    }

    let flutter_plugins_dependencies = project_dir.join(".flutter-plugins-dependencies");
    if flutter_plugins_dependencies.exists() {
        dependency_files.push(flutter_plugins_dependencies);
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
    fn test_dart_fingerprint() {
        let temp = tempdir().unwrap();
        let lib_dir = temp.path().join("lib");
        fs::create_dir_all(&lib_dir).unwrap();

        let dart_file = lib_dir.join("main.dart");
        fs::write(&dart_file, "void main() { print('Hello'); }").unwrap();

        let pubspec_file = temp.path().join("pubspec.yaml");
        fs::write(&pubspec_file, "name: test_app").unwrap();

        let fingerprint = compute_dart_fingerprint(
            temp.path(),
            "3.0.0",
            &None,
            &DartProjectType::Dart,
        ).unwrap();

        assert!(!fingerprint.is_empty());
        assert_eq!(fingerprint.len(), 64); // blake3 hash length
    }

    #[test]
    fn test_fingerprint_changes_with_source() {
        let temp = tempdir().unwrap();
        let lib_dir = temp.path().join("lib");
        fs::create_dir_all(&lib_dir).unwrap();

        let dart_file = lib_dir.join("main.dart");
        fs::write(&dart_file, "void main() { print('Hello'); }").unwrap();

        let fp1 = compute_dart_fingerprint(
            temp.path(),
            "3.0.0",
            &None,
            &DartProjectType::Dart,
        ).unwrap();

        fs::write(&dart_file, "void main() { print('Goodbye'); }").unwrap();

        let fp2 = compute_dart_fingerprint(
            temp.path(),
            "3.0.0",
            &None,
            &DartProjectType::Dart,
        ).unwrap();

        assert_ne!(fp1, fp2);
    }
}
