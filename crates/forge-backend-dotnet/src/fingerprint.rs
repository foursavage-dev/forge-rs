#![forbid(unsafe_code)]

use crate::DotnetBackendError;
use crate::config::DotnetTargetFramework;
use std::path::Path;
use std::fs;

pub fn compute_dotnet_fingerprint(
    project_dir: &Path,
    dotnet_version: &str,
    target_framework: &DotnetTargetFramework,
) -> Result<String, DotnetBackendError> {
    let mut hasher = blake3::Hasher::new();

    // Include toolchain version
    hasher.update(dotnet_version.as_bytes());

    // Include target framework
    hasher.update(target_framework.as_str().as_bytes());

    // Include source files
    let source_files = collect_source_files(project_dir)?;
    for file in &source_files {
        if let Ok(content) = fs::read(file) {
            hasher.update(&content);
        }
    }

    // Include project files
    let project_files = collect_project_files(project_dir)?;
    for file in &project_files {
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

fn collect_source_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, DotnetBackendError> {
    let mut source_files = Vec::new();
    let extensions = [".cs", ".fs", ".vb", ".xaml"];
    
    let source_dirs = [
        project_dir.join("src"),
        project_dir.join("App"),
        project_dir.join("Views"),
        project_dir.join("ViewModels"),
        project_dir.join("Models"),
        project_dir.join("Services"),
        project_dir.join("Controllers"),
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

fn collect_project_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, DotnetBackendError> {
    let mut project_files = Vec::new();
    let project_extensions = [".csproj", ".fsproj", ".vbproj", ".sln"];
    
    if let Ok(entries) = fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if project_extensions.iter().any(|e| *e == ext) {
                    project_files.push(path);
                }
            }
        }
    }

    Ok(project_files)
}

fn collect_dependency_files(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, DotnetBackendError> {
    let mut dependency_files = Vec::new();

    // NuGet packages.config
    let packages_config = project_dir.join("packages.config");
    if packages_config.exists() {
        dependency_files.push(packages_config);
    }

    // NuGet project.json (old format)
    let project_json = project_dir.join("project.json");
    if project_json.exists() {
        dependency_files.push(project_json);
    }

    // Directory.Build.props
    let directory_build_props = project_dir.join("Directory.Build.props");
    if directory_build_props.exists() {
        dependency_files.push(directory_build_props);
    }

    // NuGet.Config
    let nuget_config = project_dir.join("NuGet.Config");
    if nuget_config.exists() {
        dependency_files.push(nuget_config);
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
    fn test_dotnet_fingerprint() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let cs_file = src_dir.join("Program.cs");
        fs::write(&cs_file, "using System; class Program { static void Main() {} }").unwrap();

        let csproj_file = temp.path().join("TestApp.csproj");
        fs::write(&csproj_file, "<Project></Project>").unwrap();

        let fingerprint = compute_dotnet_fingerprint(
            temp.path(),
            "8.0.0",
            &DotnetTargetFramework::Net8_0,
        ).unwrap();

        assert!(!fingerprint.is_empty());
        assert_eq!(fingerprint.len(), 64); // blake3 hash length
    }

    #[test]
    fn test_fingerprint_changes_with_source() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let cs_file = src_dir.join("Program.cs");
        fs::write(&cs_file, "using System; class Program { static void Main() {} }").unwrap();

        let fp1 = compute_dotnet_fingerprint(
            temp.path(),
            "8.0.0",
            &DotnetTargetFramework::Net8_0,
        ).unwrap();

        fs::write(&cs_file, "using System; class Program { static void Main() { Console.WriteLine(); } }").unwrap();

        let fp2 = compute_dotnet_fingerprint(
            temp.path(),
            "8.0.0",
            &DotnetTargetFramework::Net8_0,
        ).unwrap();

        assert_ne!(fp1, fp2);
    }
}
