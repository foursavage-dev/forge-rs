//! Executors turn `Task`s into `TaskOutcome`s.
//!
//! The `TaskExecutor` trait is the boundary between the scheduler and any
//! mechanism that can build a task: process execution, remote execution,
//! local caching, mock executors in tests. The scheduler only ever sees
//! `&dyn`-style `TaskExecutor`s, so behavior can be layered
//! (e.g. `CachingExecutor` wrapping a `ProcessExecutor`).

use std::process::Command;
use std::time::Instant;

use crate::task::{Task, TaskOutcome, TaskStatus};

/// Errors that can occur when executing a task.
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    /// The process could not be spawned at all.
    #[error("failed to spawn `{command}`: {source}")]
    Spawn {
        /// The command line that failed to spawn.
        command: String,
        /// The underlying OS error.
        #[source]
        source: std::io::Error,
    },
    /// The executor could not record the outcome (I/O failure).
    #[allow(dead_code)]
    #[error("failed to record output of `{command}`: {source}")]
    Record {
        command: String,
        #[source]
        source: std::io::Error,
    },
}

/// Executes tasks on this machine. Implementations must be `Send + Sync` so
/// the scheduler can run tasks concurrently.
pub trait TaskExecutor: Send + Sync {
    /// Execute a single task and report its outcome.
    ///
    /// Must not panic (`Scheduler::run` treats panics as a failed task, but
    /// implementations shouldn't rely on that).
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError>;
}

/// The standard executor: run the task's command as a child process, capture
/// stdout/stderr, and classify the outcome by exit code.
#[derive(Debug, Clone)]
pub struct ProcessExecutor {
    /// Print task output lines to stderr as they complete (for `-v`).
    pub verbose: bool,
}

impl ProcessExecutor {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl Default for ProcessExecutor {
    fn default() -> Self {
        Self::new(false)
    }
}

impl TaskExecutor for ProcessExecutor {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        let start = Instant::now();
        let mut command: Command = task.spec.to_std_command();
        let output = match command.output() {
            Ok(output) => output,
            Err(source) => {
                return Err(ExecutorError::Spawn {
                    command: task.spec.command_line(),
                    source,
                });
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let status = if output.status.success() {
            TaskStatus::Executed
        } else {
            TaskStatus::Failed
        };
        if self.verbose {
            if !stdout.is_empty() {
                eprint!("{stdout}");
            }
            if !stderr.is_empty() {
                eprint!("{stderr}");
            }
        }
        Ok(TaskOutcome {
            status,
            exit_code: output.status.code(),
            stdout,
            stderr,
            duration: start.elapsed(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandSpec;
    use crate::task::Task;

    fn task_for(spec: CommandSpec) -> Task {
        Task::new("test".to_string(), spec.command_line(), spec)
    }

    #[test]
    fn runs_a_successful_command() {
        let executor = ProcessExecutor::new(false);
        let task = task_for(CommandSpec::new("cargo").arg("--version"));
        let outcome = executor.execute(&task).expect("cargo must spawn");
        assert_eq!(outcome.status, TaskStatus::Executed);
        assert_eq!(outcome.exit_code, Some(0));
        assert!(outcome.stdout.contains("cargo"));
    }

    #[test]
    fn captures_a_failed_command() {
        let executor = ProcessExecutor::new(false);
        let task = task_for(
            CommandSpec::new("cargo")
                .arg("metadata")
                .arg("--manifest-path")
                .arg("definitely-missing-path/Cargo.toml"),
        );
        let outcome = executor.execute(&task).expect("cargo must spawn");
        assert_eq!(outcome.status, TaskStatus::Failed);
        assert_ne!(outcome.exit_code, Some(0));
    }

    #[test]
    fn reports_a_missing_program() {
        let executor = ProcessExecutor::new(false);
        let task = task_for(CommandSpec::new("forge-definitely-not-a-real-program-9f3a"));
        let error = executor.execute(&task).expect_err("spawn must fail");
        assert!(matches!(error, ExecutorError::Spawn { .. }));
    }
}
