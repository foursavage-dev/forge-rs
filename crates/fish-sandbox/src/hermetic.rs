use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxPlatform {
    LinuxBubblewrap,
    MacOSSandboxExec,
    WindowsJobObject,
    FallbackRestricted,
}

#[derive(Debug, Clone)]
pub struct HermeticProcessSandbox {
    pub platform: SandboxPlatform,
    pub root_dir: PathBuf,
    pub writable_dirs: Vec<PathBuf>,
    pub allow_network: bool,
    pub max_memory_bytes: u64,
}

impl HermeticProcessSandbox {
    pub fn auto_configure(root_dir: PathBuf, out_dir: PathBuf) -> Self {
        let platform = if cfg!(target_os = "linux") {
            SandboxPlatform::LinuxBubblewrap
        } else if cfg!(target_os = "macos") {
            SandboxPlatform::MacOSSandboxExec
        } else if cfg!(target_os = "windows") {
            SandboxPlatform::WindowsJobObject
        } else {
            SandboxPlatform::FallbackRestricted
        };

        Self {
            platform,
            root_dir,
            writable_dirs: vec![out_dir, std::env::temp_dir()],
            allow_network: false,
            max_memory_bytes: 4 * 1024 * 1024 * 1024,
        }
    }

    pub fn wrap_command_args(&self, executable: &str, args: &[String]) -> (String, Vec<String>) {
        match self.platform {
            SandboxPlatform::LinuxBubblewrap => {
                let mut bwrap_args = vec![
                    "--unshare-all".to_string(),
                    "--ro-bind".to_string(),
                    "/".to_string(),
                    "/".to_string(),
                    "--dev".to_string(),
                    "/dev".to_string(),
                    "--proc".to_string(),
                    "/proc".to_string(),
                ];
                for w in &self.writable_dirs {
                    bwrap_args.push("--bind".to_string());
                    bwrap_args.push(w.to_string_lossy().to_string());
                    bwrap_args.push(w.to_string_lossy().to_string());
                }
                bwrap_args.push(executable.to_string());
                bwrap_args.extend_from_slice(args);
                ("bwrap".to_string(), bwrap_args)
            }
            SandboxPlatform::MacOSSandboxExec => {
                let mut sb_args = vec![
                    "-n".to_string(),
                    "-p".to_string(),
                    "(version 1)(deny default)(allow process-exec)(allow file-read*)".to_string(),
                    executable.to_string(),
                ];
                sb_args.extend_from_slice(args);
                ("sandbox-exec".to_string(), sb_args)
            }
            SandboxPlatform::WindowsJobObject | SandboxPlatform::FallbackRestricted => {
                (executable.to_string(), args.to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hermetic_sandbox_wrapping() {
        let sb = HermeticProcessSandbox {
            platform: SandboxPlatform::LinuxBubblewrap,
            root_dir: PathBuf::from("/workspace"),
            writable_dirs: vec![PathBuf::from("/workspace/target")],
            allow_network: false,
            max_memory_bytes: 1024 * 1024,
        };

        let (cmd, args) = sb.wrap_command_args("gcc", &["-c".to_string(), "main.c".to_string()]);
        assert_eq!(cmd, "bwrap");
        assert!(args.contains(&"--unshare-all".to_string()));
        assert!(args.contains(&"gcc".to_string()));
    }
}
