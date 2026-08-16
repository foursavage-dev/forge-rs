use std::path::Path;

use forge_core::{FingerprintUtils, DEFAULT_EXCLUDED_DIRS};
use crate::DartBackendError;
use crate::config::DartProjectType;

pub fn compute_dart_fingerprint(
    project_dir: &Path,
    dart_version: &str,
    flutter_version: &Option<String>,
    project_type: &DartProjectType,
) -> Result<String, DartBackendError> {
    let mut hasher = blake3::Hasher::new();

    hasher.update(dart_version.as_bytes());
    if let Some(flutter_ver) = flutter_version {
        hasher.update(flutter_ver.as_bytes());
    }

    match project_type {
        DartProjectType::Dart => {
            hasher.update(b"dart");
        }
        DartProjectType::Flutter => {
            hasher.update(b"flutter");
        }
    }

    let extensions = &["dart", "yaml", "json", "arb"];
    FingerprintUtils::hash_directory_with_extensions(
        project_dir,
        extensions,
        DEFAULT_EXCLUDED_DIRS,
        &mut hasher,
    )
    .map_err(DartBackendError::Io)?;

    for name in &[
        "pubspec.yaml",
        "pubspec.lock",
        "analysis_options.yaml",
        ".metadata",
        ".packages",
    ] {
        let path = project_dir.join(name);
        if path.exists() {
            let _ = FingerprintUtils::hash_file_into(&path, &mut hasher);
        }
    }

    let pkg_config = project_dir.join(".dart_tool").join("package_config.json");
    if pkg_config.exists() {
        let _ = FingerprintUtils::hash_file_into(&pkg_config, &mut hasher);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
        assert_eq!(fingerprint.len(), 64);
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
