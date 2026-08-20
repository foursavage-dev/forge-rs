use std::path::Path;

use crate::JavaBackendError;
use crate::config::JavaBuildSystem;
use fish_core::{DEFAULT_EXCLUDED_DIRS, FingerprintUtils};

pub fn compute_java_fingerprint(
    project_dir: &Path,
    java_version: &str,
    kotlin_version: &Option<String>,
    build_system: &JavaBuildSystem,
) -> Result<String, JavaBackendError> {
    let mut hasher = blake3::Hasher::new();

    hasher.update(java_version.as_bytes());
    if let Some(kotlin_ver) = kotlin_version {
        hasher.update(kotlin_ver.as_bytes());
    }

    match build_system {
        JavaBuildSystem::Maven => {
            hasher.update(b"maven");
        }
        JavaBuildSystem::Gradle => {
            hasher.update(b"gradle");
        }
    }

    let java_extensions = ["java", "kt", "kts", "scala", "groovy"];
    FingerprintUtils::hash_directory_with_extensions(
        project_dir,
        &java_extensions,
        DEFAULT_EXCLUDED_DIRS,
        &mut hasher,
    )
    .map_err(JavaBackendError::Io)?;

    match build_system {
        JavaBuildSystem::Maven => {
            for name in &["pom.xml", "pom.properties"] {
                let path = project_dir.join(name);
                if path.exists() {
                    let _ = FingerprintUtils::hash_file_into(&path, &mut hasher);
                }
            }
        }
        JavaBuildSystem::Gradle => {
            for name in &[
                "build.gradle",
                "build.gradle.kts",
                "settings.gradle",
                "settings.gradle.kts",
                "gradle.properties",
            ] {
                let path = project_dir.join(name);
                if path.exists() {
                    let _ = FingerprintUtils::hash_file_into(&path, &mut hasher);
                }
            }
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
    fn test_java_fingerprint() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src").join("main").join("java");
        fs::create_dir_all(&src_dir).unwrap();

        let java_file = src_dir.join("Test.java");
        fs::write(&java_file, "public class Test {}").unwrap();

        let pom_file = temp.path().join("pom.xml");
        fs::write(&pom_file, "<project></project>").unwrap();

        let fingerprint =
            compute_java_fingerprint(temp.path(), "openjdk 17", &None, &JavaBuildSystem::Maven)
                .unwrap();

        assert!(!fingerprint.is_empty());
        assert_eq!(fingerprint.len(), 64);
    }

    #[test]
    fn test_fingerprint_changes_with_source() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src").join("main").join("java");
        fs::create_dir_all(&src_dir).unwrap();

        let java_file = src_dir.join("Test.java");
        fs::write(&java_file, "public class Test {}").unwrap();

        let fp1 =
            compute_java_fingerprint(temp.path(), "openjdk 17", &None, &JavaBuildSystem::Maven)
                .unwrap();

        fs::write(&java_file, "public class Test { int x; }").unwrap();

        let fp2 =
            compute_java_fingerprint(temp.path(), "openjdk 17", &None, &JavaBuildSystem::Maven)
                .unwrap();

        assert_ne!(fp1, fp2);
    }
}
