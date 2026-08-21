#![forbid(unsafe_code)]

//! Async process executor for improved I/O performance
//!
//! This module provides an async executor that reduces blocking during
//! process execution and output handling.
//!
//! Performance optimizations:
//! - Async process spawning with Tokio
//! - Non-blocking stdout/stderr capture
//! - Better resource utilization during I/O-bound operations

use crate::executor::ExecutorError;
use crate::task::{Task, TaskOutcome, TaskStatus};
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command as TokioCommand;

/// Async process executor
pub struct AsyncProcessExecutor {
    pub verbose: bool,
    pub timeout: Option<Duration>,
}

impl AsyncProcessExecutor {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            timeout: None,
        }
    }

    pub fn with_timeout(verbose: bool, timeout: Option<Duration>) -> Self {
        Self { verbose, timeout }
    }

    /// Execute a task asynchronously
    pub async fn execute_async(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        let start = Instant::now();
        let mut command: TokioCommand = self.spec_to_tokio_command(&task.spec);

        let output = if let Some(timeout) = self.timeout {
            self.run_with_timeout_async(&mut command, timeout).await
        } else {
            self.run_async(&mut command).await
        };

        let output = match output {
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

    /// Convert CommandSpec to Tokio Command
    fn spec_to_tokio_command(&self, spec: &crate::CommandSpec) -> TokioCommand {
        let mut command = TokioCommand::new(&spec.program);
        command.args(&spec.args);

        // `env_clear` was previously ignored on the async path, silently
        // inheriting the full environment even when the caller asked for an
        // empty one. Mirror the synchronous executor's behaviour.
        if spec.env_clear {
            command.env_clear();
        }
        for (key, value) in &spec.env {
            command.env(key, value);
        }

        if let Some(cwd) = &spec.cwd {
            command.current_dir(cwd);
        }

        // If this command is dropped (e.g. a timeout cancels the run), kill
        // the child instead of orphaning a long-lived build process.
        command.kill_on_drop(true);

        command
    }

    /// Run command asynchronously with basic output capture
    async fn run_async(
        &self,
        command: &mut TokioCommand,
    ) -> Result<std::process::Output, std::io::Error> {
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        let mut child = command.spawn()?;

        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");

        // Use Tokio's async I/O for reading
        let (stdout_result, stderr_result) = tokio::join!(
            async {
                let mut stdout = stdout;
                let mut buf = Vec::new();
                stdout.read_to_end(&mut buf).await.map(|_| buf)
            },
            async {
                let mut stderr = stderr;
                let mut buf = Vec::new();
                stderr.read_to_end(&mut buf).await.map(|_| buf)
            }
        );

        let stdout = stdout_result?;
        let stderr = stderr_result?;

        let status = child.wait().await?;

        Ok(std::process::Output {
            status,
            stdout,
            stderr,
        })
    }

    /// Run command with timeout asynchronously
    async fn run_with_timeout_async(
        &self,
        command: &mut TokioCommand,
        timeout: Duration,
    ) -> Result<std::process::Output, std::io::Error> {
        tokio::time::timeout(timeout, self.run_async(command))
            .await
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("timed out after {timeout:?}"),
                )
            })?
    }
}

impl Default for AsyncProcessExecutor {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandSpec;
    use crate::task::{Task, TaskStatus};
    use std::time::{Duration, Instant};

    fn task_for(spec: CommandSpec) -> Task {
        Task::new("test".to_string(), spec.command_line(), spec)
    }

    #[test]
    fn test_async_executor_basic() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let executor = AsyncProcessExecutor::new(false);
            let task = task_for(CommandSpec::new("cargo").arg("--version"));
            let outcome = executor
                .execute_async(&task)
                .await
                .expect("cargo must spawn");
            assert_eq!(outcome.status, TaskStatus::Executed);
            assert_eq!(outcome.exit_code, Some(0));
            assert!(outcome.stdout.contains("cargo"));
        });
    }

    #[test]
    fn test_async_executor_timeout() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let executor =
                AsyncProcessExecutor::with_timeout(false, Some(Duration::from_millis(100)));
            let (prog, args) = if cfg!(windows) {
                (
                    "powershell",
                    vec!["-Command".to_string(), "Start-Sleep -Seconds 2".to_string()],
                )
            } else {
                ("sleep", vec!["2".to_string()])
            };
            let task = task_for(CommandSpec::new(prog).args(args));

            let start = Instant::now();
            let error = executor.execute_async(&task).await.unwrap_err();
            assert!(error.to_string().contains("timed out"), "error: {error:?}");
            assert!(
                start.elapsed() < Duration::from_secs(1),
                "the child must be killed, not waited on"
            );
        });
    }
}
