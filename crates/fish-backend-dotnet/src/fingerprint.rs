use std::path::Path;

use crate::DotnetBackendError;
use crate::config::DotnetTargetFramework;
use fish_core::{DEFAULT_EXCLUDED_DIRS, FingerprintUtils};

pub fn compute_dotnet_fingerprint(
    project_dir: &Path,
    dotnet_version: &str,
    target_framework: &DotnetTargetFramework,
) -> Result<String, DotnetBackendError> {
    let mut hasher = blake3::Hasher::new();

    hasher.update(dotnet_version.as_bytes());
    hasher.update(target_framework.as_str().as_bytes());

    let extensions = &[
        "cs", "fs", "vb", "xaml", "csproj", "fsproj", "vbproj", "sln", "props", "targets", "json",
        "config",
    ];
    FingerprintUtils::hash_directory_with_extensions(
        project_dir,
        extensions,
        DEFAULT_EXCLUDED_DIRS,
        &mut hasher,
    )
    .map_err(DotnetBackendError::Io)?;

    for name in &[
        "Directory.Build.props",
        "Directory.Build.targets",
        "NuGet.Config",
        "global.json",
    ] {
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
    fn test_dotnet_fingerprint() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let cs_file = src_dir.join("Program.cs");
        fs::write(
            &cs_file,
            "using System; class Program { static void Main() {} }",
        )
        .unwrap();

        let csproj_file = temp.path().join("TestApp.csproj");
        fs::write(&csproj_file, "<Project></Project>").unwrap();

        let fingerprint =
            compute_dotnet_fingerprint(temp.path(), "8.0.0", &DotnetTargetFramework::Net8_0)
                .unwrap();

        assert!(!fingerprint.is_empty());
        assert_eq!(fingerprint.len(), 64);
    }

    #[test]
    fn test_fingerprint_changes_with_source() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let cs_file = src_dir.join("Program.cs");
        fs::write(
            &cs_file,
            "using System; class Program { static void Main() {} }",
        )
        .unwrap();

        let fp1 = compute_dotnet_fingerprint(temp.path(), "8.0.0", &DotnetTargetFramework::Net8_0)
            .unwrap();

        fs::write(
            &cs_file,
            "using System; class Program { static void Main() { Console.WriteLine(); } }",
        )
        .unwrap();

        let fp2 = compute_dotnet_fingerprint(temp.path(), "8.0.0", &DotnetTargetFramework::Net8_0)
            .unwrap();

        assert_ne!(fp1, fp2);
    }
}
