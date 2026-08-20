use std::fs;
use std::path::{Path, PathBuf};

use crate::command::CommandSpec;

pub struct ResponseFileWriter;

impl ResponseFileWriter {
    pub const WINDOWS_MAX_CMD_LEN: usize = 4096;
    pub const POSIX_MAX_CMD_LEN: usize = 32768;

    pub fn should_use_response_file(args: &[String]) -> bool {
        let total_len: usize = args.iter().map(|a| a.len() + 1).sum();
        let limit = if cfg!(windows) {
            Self::WINDOWS_MAX_CMD_LEN
        } else {
            Self::POSIX_MAX_CMD_LEN
        };
        total_len > limit || args.len() > 100
    }

    pub fn write_response_file(dir: &Path, args: &[String]) -> std::io::Result<PathBuf> {
        fs::create_dir_all(dir)?;
        let hash = fastrand::u64(..);
        let path = dir.join(format!("fish_args_{hash:016x}.rsp"));
        let mut content = String::new();
        for arg in args {
            if arg.contains(' ') || arg.contains('\t') || arg.contains('"') {
                content.push('"');
                content.push_str(&arg.replace('\\', "\\\\").replace('"', "\\\""));
                content.push('"');
            } else {
                content.push_str(arg);
            }
            content.push('\n');
        }
        fs::write(&path, content)?;
        Ok(path)
    }

    pub fn adapt_command_for_response_file(
        spec: &mut CommandSpec,
        temp_dir: &Path,
    ) -> std::io::Result<Option<PathBuf>> {
        if !Self::should_use_response_file(&spec.args) {
            return Ok(None);
        }
        let rsp_path = Self::write_response_file(temp_dir, &spec.args)?;
        spec.args.clear();
        spec.args.push(format!("@{}", rsp_path.to_string_lossy()));
        Ok(Some(rsp_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_should_use_response_file_thresholds() {
        let small_args = vec!["--flag".to_string(), "target".to_string()];
        assert!(!ResponseFileWriter::should_use_response_file(&small_args));

        let large_args: Vec<String> = (0..150).map(|i| format!("arg_{i}")).collect();
        assert!(ResponseFileWriter::should_use_response_file(&large_args));
    }

    #[test]
    fn test_write_response_file_and_adapt() {
        let temp = tempdir().unwrap();
        let mut spec = CommandSpec::new("rustc");
        for i in 0..200 {
            spec.args.push(format!("--extern=crate_{i}"));
        }
        let rsp = ResponseFileWriter::adapt_command_for_response_file(&mut spec, temp.path())
            .unwrap()
            .expect("should create rsp");

        assert!(rsp.exists());
        assert_eq!(spec.args.len(), 1);
        assert!(spec.args[0].starts_with('@'));

        let content = fs::read_to_string(&rsp).unwrap();
        assert!(content.contains("--extern=crate_0"));
        assert!(content.contains("--extern=crate_199"));
    }
}
