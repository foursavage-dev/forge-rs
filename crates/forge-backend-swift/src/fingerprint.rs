use std::path::Path;

use forge_core::{FingerprintUtils, DEFAULT_EXCLUDED_DIRS};
use crate::SwiftBackendError;
use crate::config::SwiftPlatform;

pub fn compute_swift_fingerprint(
    project_dir: &Path,
    swift_version: &str,
    platform: &SwiftPlatform,
) -> Result<String, SwiftBackendError> {
    let mut hasher = blake3::Hasher::new();

    hasher.update(swift_version.as_bytes());
    hasher.update(platform.as_str().as_bytes());

    let extensions = &["swift", "m", "mm", "h", "hpp", "c", "cpp", "modulemap"];
    FingerprintUtils::hash_directory_with_extensions(
        project_dir,
        extensions,
        DEFAULT_EXCLUDED_DIRS,
        &mut hasher,
    )
    .map_err(SwiftBackendError::Io)?;

    for name in &["Package.swift", "Package.resolved", "Podfile", "Podfile.lock", "Cartfile", "Cartfile.resolved"] {
        let path = project_dir.join(name);
        if path.exists() {
            let _ = FingerprintUtils::hash_file_into(&path, &mut hasher);
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
        assert_eq!(fingerprint.len(), 64);
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
