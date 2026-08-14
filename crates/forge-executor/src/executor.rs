//! Executors turn `Task`s into `TaskOutcome`s.
//!
//! The `TaskExecutor` trait is the boundary between the scheduler and any
//! mechanism that can build a task: process execution, remote execution,
//! local caching, mock executors in tests. The scheduler only ever sees
//! `&dyn`-style `TaskExecutor`s, so behavior can be layered
//! (e.g. `CachingExecutor` wrapping a `ProcessExecutor`).

use std::process::Command;
use std::time::{Duration, Instant};

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
    /// Maximum time a task may run before it is killed and reported as
    /// failed (`None` = no limit).
    pub timeout: Option<Duration>,
}

impl ProcessExecutor {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            timeout: None,
        }
    }

    /// The maximum time a task may run before it is killed.
    pub fn with_timeout(verbose: bool, timeout: Option<Duration>) -> Self {
        Self { verbose, timeout }
    }
}

impl Default for ProcessExecutor {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Kill `child` and, on Windows, its whole descendant tree.
///
/// Cargo spawns rustc, which spawns build scripts; terminating only the
/// direct child would leave those grandchildren running as orphans.
fn kill_process_tree(child: &mut std::process::Child) {
    #[cfg(windows)]
    {
        let pid = child.id();
        let _ = std::process::Command::new("taskkill")
            .arg("/PID")
            .arg(pid.to_string())
            .arg("/T")
            .arg("/F")
            .status();
    }
    #[cfg(not(windows))]
    {
        // Without a separate process group there is no safe tree-kill
        // (the child shares forge's group); kill the direct child.
        let _ = child.kill();
    }
    let _ = child.wait();
}

/// Run `command`, killing it if it outlives `timeout`.
///
/// `Command::output()` blocks until the child exits, so a hung task would
/// hang the whole build. This path spawns the child with piped output,
/// drains the pipes on reader threads (so a chatty child can never
/// deadlock the timeout), and polls `try_wait` against a deadline. On
/// timeout the child is killed and reaped; the reader threads are
/// detached and finish whenever the pipes close.
fn run_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> Result<std::process::Output, std::io::Error> {
    use std::io::Read;
    use std::process::Stdio;

    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let mut stdout = child.stdout.take().expect("piped stdout is present");
    let mut stderr = child.stderr.take().expect("piped stderr is present");
    let out_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });
    let err_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf);
        buf
    });

    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = out_thread.join().unwrap_or_default();
                let stderr = err_thread.join().unwrap_or_default();
                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) if Instant::now() >= deadline => {
                kill_process_tree(&mut child);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("timed out after {timeout:?}"),
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(source) => return Err(source),
        }
    }
}

impl TaskExecutor for ProcessExecutor {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        let start = Instant::now();
        let mut command: Command = task.spec.to_std_command();
        let output = match self.timeout {
            Some(timeout) => run_with_timeout(&mut command, timeout),
            None => command.output(),
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

    #[test]
    fn kills_a_task_that_exceeds_the_timeout() {
        let executor = ProcessExecutor::with_timeout(false, Some(Duration::from_millis(100)));
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
        let error = executor.execute(&task).unwrap_err();
        assert!(error.to_string().contains("timed out"), "error: {error:?}");
        assert!(
            start.elapsed() < Duration::from_secs(1),
            "the child must be killed, not waited on"
        );
    }
}
