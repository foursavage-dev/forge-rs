use std::process::Command;
use std::time::{Duration, Instant};

use crate::task::{Task, TaskOutcome, TaskStatus};

#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("failed to spawn `{command}`: {source}")]
    Spawn {
        command: String,

        #[source]
        source: std::io::Error,
    },

    #[error("failed to record output of `{command}`: {source}")]
    Record {
        command: String,
        #[source]
        source: std::io::Error,
    },
}

pub trait TaskExecutor: Send + Sync {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError>;
}

#[derive(Debug, Clone)]
pub struct ProcessExecutor {
    pub verbose: bool,

    pub timeout: Option<Duration>,
}

impl ProcessExecutor {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            timeout: None,
        }
    }

    pub fn with_timeout(verbose: bool, timeout: Option<Duration>) -> Self {
        Self { verbose, timeout }
    }
}

impl Default for ProcessExecutor {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(windows)]
fn kill_process_tree(child: &mut std::process::Child) {
    let pid = child.id();
    let _ = std::process::Command::new("taskkill")
        .arg("/PID")
        .arg(pid.to_string())
        .arg("/T")
        .arg("/F")
        .status();
    let _ = child.wait();
}

#[cfg(all(unix, not(target_os = "macos")))]
fn kill_process_tree(child: &mut std::process::Child) {
    let pid = child.id();

    // Kill the whole descendant tree (not just the direct child), so a build
    // command that spawned subprocesses (make -> gcc) cannot leave orphans
    // behind after a timeout. `/proc/<pid>/task/<pid>/children` lists direct
    // children; walk it depth-first and signal leaves first.
    fn kill_descendants(pid: u32) {
        let children_path = format!("/proc/{pid}/task/{pid}/children");
        if let Ok(contents) = std::fs::read_to_string(&children_path) {
            for child_pid in contents
                .split_whitespace()
                .filter_map(|s| s.parse::<u32>().ok())
            {
                kill_descendants(child_pid);
                let _ = std::process::Command::new("kill")
                    .args(["-KILL", &child_pid.to_string()])
                    .status();
            }
        }
    }

    kill_descendants(pid);
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(all(unix, target_os = "macos"))]
fn kill_process_tree(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn run_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> Result<std::process::Output, std::io::Error> {
    use std::io::Read;
    use std::process::Stdio;
    use wait_timeout::ChildExt;

    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let mut stdout = child.stdout.take().expect("piped stdout is present");
    let mut stderr = child.stderr.take().expect("piped stderr is present");

    // Drain both pipes on background threads. `wait_timeout` does not read the
    // pipes, so a child that writes more than the pipe buffer (64 KiB) would
    // otherwise block forever on `write`, triggering a spurious timeout.
    let stdout_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf);
        buf
    });

    let status_code = match child.wait_timeout(timeout)? {
        Some(status) => status,
        None => {
            kill_process_tree(&mut child);
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!("timed out after {timeout:?}"),
            ));
        }
    };

    let out_buf = stdout_reader.join().unwrap_or_default();
    let err_buf = stderr_reader.join().unwrap_or_default();

    Ok(std::process::Output {
        status: status_code,
        stdout: out_buf,
        stderr: err_buf,
    })
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
        let stdout = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(e) => String::from_utf8_lossy(&e.into_bytes()).into_owned(),
        };
        let stderr = match String::from_utf8(output.stderr) {
            Ok(s) => s,
            Err(e) => String::from_utf8_lossy(&e.into_bytes()).into_owned(),
        };
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

impl<T: ?Sized + TaskExecutor> TaskExecutor for Box<T> {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        (**self).execute(task)
    }
}

impl<T: ?Sized + TaskExecutor> TaskExecutor for &T {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        (**self).execute(task)
    }
}

impl<T: ?Sized + TaskExecutor> TaskExecutor for std::sync::Arc<T> {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        (**self).execute(task)
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
        let task = task_for(CommandSpec::new("fish-definitely-not-a-real-program-9f3a"));
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
