use std::path::Path;

use crate::config::{PackageManager, TsTaskSpec};
use forge_executor::CommandSpec;

#[derive(Debug, Clone, Default)]
pub struct TsToolchain;

impl TsToolchain {
    pub fn new() -> Self {
        Self
    }

    /// Windows package managers (npm, pnpm, yarn, npx) are `.cmd` shims, and
    /// `std::process::Command` cannot find a bare name without an extension.
    /// Resolve the first `{program}.{exe,cmd,bat}` on `PATH` so the task can
    /// actually spawn; fall back to the bare name on Unix or when nothing
    /// matches, letting the OS produce the real error.
    pub fn resolve_program(program: &str) -> String {
        if !cfg!(windows) {
            return program.to_string();
        }
        if Path::new(program).extension().is_some() || program.contains(['/', '\\']) {
            return program.to_string();
        }
        let Some(path) = std::env::var_os("PATH") else {
            return program.to_string();
        };
        for dir in std::env::split_paths(&path) {
            for ext in ["exe", "cmd", "bat"] {
                let candidate = dir.join(format!("{program}.{ext}"));
                if candidate.is_file() {
                    return candidate.to_string_lossy().to_string();
                }
            }
        }
        program.to_string()
    }

    pub fn build_command(&self, task: &TsTaskSpec, pm: PackageManager, root: &Path) -> CommandSpec {
        let program = task
            .command
            .clone()
            .unwrap_or_else(|| Self::resolve_program(pm.executable()));

        let mut spec = CommandSpec::new(program);
        for arg in &task.args {
            spec = spec.arg(arg);
        }
        spec.cwd(root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_program_finds_command_shims_on_windows() {
        if !cfg!(windows) {
            return;
        }
        let resolved = TsToolchain::resolve_program("cmd");
        assert!(resolved.ends_with("cmd.exe"), "got: {resolved}");
    }

    #[test]
    fn resolve_program_leaves_explicit_paths_alone() {
        let resolved = TsToolchain::resolve_program("C:\\tools\\npm.cmd");
        assert_eq!(resolved, "C:\\tools\\npm.cmd");
    }
}
