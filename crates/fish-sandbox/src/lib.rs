#![forbid(unsafe_code)]

pub mod microvm;

pub mod env;
pub mod executor;
pub mod isolation;
pub mod tracer;

pub use env::{EnvPolicy, sanitize_env};
pub use executor::{SandboxConfig, SandboxedExecutor};
pub use isolation::{FsPolicy, SandboxWorkspace};
pub use tracer::{HermeticTraceResult, SyscallTracer};

#[cfg(test)]
mod tests {
    use super::*;
    use fish_executor::{CommandSpec, ProcessExecutor, Task, TaskExecutor};
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_hermetic_env_sanitization() {
        let mut extra = HashMap::new();
        extra.insert("CUSTOM_VAR".to_string(), "custom_val".to_string());

        let env = sanitize_env(&EnvPolicy::Hermetic, &extra);

        assert_eq!(env.get("LANG").map(|s| s.as_str()), Some("C"));
        assert_eq!(env.get("LC_ALL").map(|s| s.as_str()), Some("C"));
        assert_eq!(env.get("TZ").map(|s| s.as_str()), Some("UTC"));
        assert_eq!(
            env.get("CUSTOM_VAR").map(|s| s.as_str()),
            Some("custom_val")
        );
    }

    #[test]
    fn test_sandboxed_executor_runs_task() {
        let temp = tempdir().unwrap();
        let process = ProcessExecutor::new(false);
        let config = SandboxConfig::default();
        let executor = SandboxedExecutor::new(process, config);

        let prog = if cfg!(windows) { "cmd" } else { "sh" };
        let args = if cfg!(windows) {
            vec!["/C".to_string(), "exit 0".to_string()]
        } else {
            vec!["-c".to_string(), "exit 0".to_string()]
        };

        let spec = CommandSpec::new(prog).args(args).cwd(temp.path());
        let task = Task::new("test_task", spec.command_line(), spec);

        let outcome = executor.execute(&task).unwrap();
        assert_eq!(outcome.status, fish_executor::TaskStatus::Executed);
    }

    #[test]
    fn hermetic_policy_reaches_the_child_environment() {
        let temp = tempdir().unwrap();
        let process = ProcessExecutor::new(false);
        let executor = SandboxedExecutor::new(process, SandboxConfig::default());

        let (prog, args) = if cfg!(windows) {
            ("cmd", vec!["/C".to_string(), "echo %LANG%".to_string()])
        } else {
            ("sh", vec!["-c".to_string(), "echo $LANG".to_string()])
        };

        let spec = CommandSpec::new(prog).args(args).cwd(temp.path());
        let task = Task::new("env_probe", spec.command_line(), spec);

        let outcome = executor.execute(&task).unwrap();
        assert_eq!(outcome.status, fish_executor::TaskStatus::Executed);
        assert_eq!(
            outcome.stdout.trim(),
            "C",
            "the child must see the sanitized LANG, got: {:?}",
            outcome.stdout
        );
    }
}
