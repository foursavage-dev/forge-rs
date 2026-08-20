use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PipelinedCrateCoordinator {
    pub crate_name: String,
    pub rmeta_path: PathBuf,
    pub rlib_path: PathBuf,
    pub is_ready: bool,
}

impl PipelinedCrateCoordinator {
    pub fn new(crate_name: &str, target_dir: &Path) -> Self {
        let rmeta_name = format!("lib{crate_name}.rmeta");
        let rlib_name = format!("lib{crate_name}.rlib");

        Self {
            crate_name: crate_name.to_string(),
            rmeta_path: target_dir.join(rmeta_name),
            rlib_path: target_dir.join(rlib_name),
            is_ready: false,
        }
    }

    pub fn inject_pipelining_flags(args: &mut Vec<String>) {
        if !args.iter().any(|a| a.starts_with("--emit")) {
            args.push("--emit=metadata,link".to_string());
        }
    }

    pub fn poll_rmeta_ready(&mut self) -> bool {
        if self.is_ready {
            return true;
        }

        if self.rmeta_path.is_file()
            && std::fs::metadata(&self.rmeta_path)
                .map(|m| m.len() > 0)
                .unwrap_or(false)
        {
            self.is_ready = true;
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipelining_flags() {
        let mut args = vec!["--crate-name".to_string(), "my_crate".to_string()];
        PipelinedCrateCoordinator::inject_pipelining_flags(&mut args);
        assert!(args.contains(&"--emit=metadata,link".to_string()));
    }

    #[test]
    fn test_rmeta_polling() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut coordinator = PipelinedCrateCoordinator::new("test_crate", temp_dir.path());
        assert!(!coordinator.poll_rmeta_ready());

        std::fs::write(&coordinator.rmeta_path, b"fake_rmeta_data").unwrap();
        assert!(coordinator.poll_rmeta_ready());
    }
}
