use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct RustcCompiler {
    pub executable: String,
    pub version: String,
    pub host_triple: String,
}

impl RustcCompiler {
    pub fn detect() -> Result<Self, String> {
        let executable = std::env::var_os("RUSTC")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("rustc"))
            .to_string_lossy()
            .into_owned();

        let output = Command::new(&executable)
            .arg("--version")
            .arg("--verbose")
            .output()
            .map_err(|e| format!("Failed to spawn `rustc`: {e}"))?;

        if !output.status.success() {
            return Err("`rustc --version` failed".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut version = "rustc unknown".to_string();
        let mut host_triple = "unknown".to_string();

        for line in stdout.lines() {
            if line.starts_with("rustc ") {
                version = line.trim().to_string();
            } else if line.starts_with("host: ") {
                host_triple = line.trim_start_matches("host: ").trim().to_string();
            }
        }

        Ok(Self {
            executable,
            version,
            host_triple,
        })
    }

    pub fn build_crate_args(
        &self,
        entry_point: &Path,
        crate_name: &str,
        crate_type: &str,
        edition: &str,
        out_dir: &Path,
        externs: &[(&str, PathBuf)],
    ) -> Vec<String> {
        let mut args = vec![
            entry_point.to_string_lossy().to_string(),
            "--crate-name".to_string(),
            crate_name.to_string(),
            "--crate-type".to_string(),
            crate_type.to_string(),
            "--edition".to_string(),
            edition.to_string(),
            "--out-dir".to_string(),
            out_dir.to_string_lossy().to_string(),
            "-C".to_string(),
            "opt-level=0".to_string(),
            "-C".to_string(),
            "debuginfo=2".to_string(),
            "-L".to_string(),
            format!("dependency={}", out_dir.to_string_lossy()),
        ];

        for (dep_name, dep_path) in externs {
            args.push("--extern".to_string());
            args.push(format!("{}={}", dep_name, dep_path.to_string_lossy()));
        }

        args
    }
}
