#![forbid(unsafe_code)]

use crate::JavaBackendError;
use crate::config::JavaBuildSystem;
use std::path::Path;
use std::fs;

pub fn compute_java_fingerprint(
    project_dir: &Path,
    java_version: &str,
    kotlin_version: &Option<String>,
    build_system: &JavaBuildSystem,
) -> Result<String, JavaBackendError> {
    let mut hasher = blake3::Hasher::new();

    // Include toolchain versions
    hasher.update(java_version.as_bytes());
    if let Some(kotlin_ver) = kotlin_version {
        hasher.update(kotlin_ver.as_bytes());
    }

    // Include build system type
    match build_system {
        JavaBuildSystem::Maven => {
            hasher.update(b"maven");
        }
        JavaBuildSystem::Gradle => {
            hasher.update(b"gradle");
        }
    }

    // Include source files
    let source_files = collect_source_files(project_dir)?;
    for file in &source_files {
        if let Ok(content) = fs::read(file) {
            hasher.update(&content);
        }
    }

    // Include build configuration
    match build_system {
        JavaBuildSystem::Maven => {
            let pom_path = project_dir.join("pom.xml");
            if let Ok(content) = fs::read(&pom_path) {
                hasher.update(&content);
            }
        }
        JavaBuildSystem::Gradle => {
            let build_gradle = project_dir.join("build.gradle");
            if build_gradle.exists() {
                if let Ok(content) = fs::read(&build_gradle) {
                    hasher.update(&content);
                }
            }
            let build_gradle_kts = project_dir.join("build.gradle.kts");
            if build_gradle_kts.exists() {
                if let Ok(content) = fs::read(&build_gradle_kts) {
                    hasher.update(&content);
                }
            }
            let settings_gradle = project_dir.join("settings.gradle");
            if settings_gradle.exists() {
                if let Ok(content) = fs::read(&settings_gradle) {
                    hasher.update(&content);
                }
            }
        }
    }

    // Include dependency files
    let dependency_files = collect_dependency_files(project_dir, build_system)?;
    for file in &dependency_files {
        if let Ok(content) = fs::read(file) {
            hasher.update(&content);
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn collect_source_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, JavaBackendError> {
    let mut source_files = Vec::new();
    let java_extensions = ["java", "kt", "scala"];
    
    let source_dirs = [
        project_dir.join("src").join("main").join("java"),
        project_dir.join("src").join("main").join("kotlin"),
        project_dir.join("src").join("main").join("scala"),
        project_dir.join("src").join("test").join("java"),
        project_dir.join("src").join("test").join("kotlin"),
        project_dir.join("src").join("test").join("scala"),
    ];

    for source_dir in &source_dirs {
        if source_dir.exists() {
            collect_files_recursive(source_dir, &java_extensions, &mut source_files);
        }
    }

    Ok(source_files)
}

fn collect_dependency_files(project_dir: &Path, build_system: &JavaBuildSystem) -> Result<Vec<std::path::PathBuf>, JavaBackendError> {
    let mut dependency_files = Vec::new();

    match build_system {
        JavaBuildSystem::Maven => {
            // Maven dependency files
            let maven_files = [
                project_dir.join("pom.xml"),
                project_dir.join("pom.properties"),
            ];
            for file in &maven_files {
                if file.exists() {
                    dependency_files.push(file.clone());
                }
            }
        }
        JavaBuildSystem::Gradle => {
            // Gradle dependency files
            let gradle_files = [
                project_dir.join("build.gradle"),
                project_dir.join("build.gradle.kts"),
                project_dir.join("settings.gradle"),
                project_dir.join("settings.gradle.kts"),
                project_dir.join("gradle.properties"),
            ];
            for file in &gradle_files {
                if file.exists() {
                    dependency_files.push(file.clone());
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
    fn test_java_fingerprint() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src").join("main").join("java");
        fs::create_dir_all(&src_dir).unwrap();

        let java_file = src_dir.join("Test.java");
        fs::write(&java_file, "public class Test {}").unwrap();

        let pom_file = temp.path().join("pom.xml");
        fs::write(&pom_file, "<project></project>").unwrap();

        let fingerprint = compute_java_fingerprint(
            temp.path(),
            "openjdk 17",
            &None,
            &JavaBuildSystem::Maven,
        ).unwrap();

        assert!(!fingerprint.is_empty());
        assert_eq!(fingerprint.len(), 64); // blake3 hash length
    }

    #[test]
    fn test_fingerprint_changes_with_source() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src").join("main").join("java");
        fs::create_dir_all(&src_dir).unwrap();

        let java_file = src_dir.join("Test.java");
        fs::write(&java_file, "public class Test {}").unwrap();

        let fp1 = compute_java_fingerprint(
            temp.path(),
            "openjdk 17",
            &None,
            &JavaBuildSystem::Maven,
        ).unwrap();

        fs::write(&java_file, "public class Test { int x; }").unwrap();

        let fp2 = compute_java_fingerprint(
            temp.path(),
            "openjdk 17",
            &None,
            &JavaBuildSystem::Maven,
        ).unwrap();

        assert_ne!(fp1, fp2);
    }
}
