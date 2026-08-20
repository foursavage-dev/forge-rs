#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

#[derive(Debug, Clone, Default)]
pub struct HermeticTraceResult {
    pub accessed_inputs: HashSet<PathBuf>,
    pub produced_outputs: HashSet<PathBuf>,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub struct SyscallTracer;

impl SyscallTracer {
    pub fn trace_execution(
        mut command: Command,
        watch_root: &Path,
    ) -> io::Result<HermeticTraceResult> {
        let before_snapshot = Self::scan_directory_timestamps(watch_root)?;
        let start_time = SystemTime::now();

        let output = command.output()?;
        let after_snapshot = Self::scan_directory_timestamps(watch_root)?;

        let mut produced_outputs = HashSet::new();
        let mut accessed_inputs = HashSet::new();

        for (path, modified) in &after_snapshot {
            if let Some(before_time) = before_snapshot.get(path) {
                if modified > before_time || *modified >= start_time {
                    produced_outputs.insert(path.clone());
                } else {
                    accessed_inputs.insert(path.clone());
                }
            } else {
                produced_outputs.insert(path.clone());
            }
        }

        Ok(HermeticTraceResult {
            accessed_inputs,
            produced_outputs,
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    fn scan_directory_timestamps(
        root: &Path,
    ) -> io::Result<std::collections::HashMap<PathBuf, SystemTime>> {
        let mut map = std::collections::HashMap::new();
        if !root.exists() {
            return Ok(map);
        }

        let mut stack = vec![root.to_path_buf()];
        while let Some(current) = stack.pop() {
            if let Ok(entries) = fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if name != ".git" && name != "target" && name != ".fish" {
                            stack.push(path);
                        }
                    } else if path.is_file()
                        && let Ok(meta) = fs::metadata(&path)
                        && let Ok(mod_time) = meta.modified()
                    {
                        map.insert(path, mod_time);
                    }
                }
            }
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_syscall_tracer_tracks_created_file() {
        let temp = tempdir().unwrap();
        let src_file = temp.path().join("source.txt");
        fs::write(&src_file, "input content").unwrap();

        let out_file = temp.path().join("output.txt");

        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.arg("/C").arg("echo generated > output.txt");
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg("echo generated > output.txt");
            c
        };
        cmd.current_dir(temp.path());

        let result = SyscallTracer::trace_execution(cmd, temp.path()).unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(out_file.exists());
        assert!(result.produced_outputs.contains(&out_file));
    }
}
