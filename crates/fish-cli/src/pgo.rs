use std::path::{Path, PathBuf};

pub struct PgoManager;

impl PgoManager {
    pub fn profile_dir(workspace_root: &Path) -> PathBuf {
        workspace_root.join("target").join("pgo_profiles")
    }

    pub fn generate_instrument_flags(profile_dir: &Path) -> Vec<String> {
        let dir_str = profile_dir.to_string_lossy();
        vec![
            format!("-Cprofile-generate={dir_str}"),
            "-Clto=thin".to_string(),
        ]
    }

    pub fn generate_optimize_flags(profile_dir: &Path) -> Vec<String> {
        let dir_str = profile_dir.to_string_lossy();
        let profdata_file = profile_dir.join("merged.profdata");
        let path_str = if profdata_file.exists() {
            profdata_file.to_string_lossy().to_string()
        } else {
            dir_str.to_string()
        };
        vec![
            format!("-Cprofile-use={path_str}"),
            "-Clto=thin".to_string(),
            "-Ccodegen-units=1".to_string(),
        ]
    }

    pub fn merge_profdata_command(profile_dir: &Path) -> Option<std::process::Command> {
        let merged_path = profile_dir.join("merged.profdata");
        let mut cmd = std::process::Command::new("llvm-profdata");
        cmd.arg("merge")
            .arg("-output")
            .arg(merged_path)
            .arg(profile_dir);
        Some(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_pgo_flag_synthesis() {
        let temp = tempdir().unwrap();
        let pgo_dir = PgoManager::profile_dir(temp.path());

        let gen_flags = PgoManager::generate_instrument_flags(&pgo_dir);
        assert!(gen_flags.iter().any(|f| f.contains("-Cprofile-generate")));
        assert!(gen_flags.iter().any(|f| f.contains("-Clto=thin")));

        let opt_flags = PgoManager::generate_optimize_flags(&pgo_dir);
        assert!(opt_flags.iter().any(|f| f.contains("-Cprofile-use")));
        assert!(opt_flags.iter().any(|f| f.contains("-Ccodegen-units=1")));
    }
}
