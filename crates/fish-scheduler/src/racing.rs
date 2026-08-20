use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use fish_executor::{ExecutorError, Task, TaskExecutor, TaskOutcome};

pub struct DynamicRacingExecutor<L, R> {
    local_executor: Arc<L>,
    remote_executor: Arc<R>,
    remote_grace_period: Duration,
}

impl<L, R> DynamicRacingExecutor<L, R>
where
    L: TaskExecutor + Send + Sync + 'static,
    R: TaskExecutor + Send + Sync + 'static,
{
    pub fn new(local: L, remote: R, remote_grace_period: Duration) -> Self {
        Self {
            local_executor: Arc::new(local),
            remote_executor: Arc::new(remote),
            remote_grace_period,
        }
    }

    pub fn execute_race(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        let (tx, rx) = crossbeam_channel::bounded::<Result<TaskOutcome, ExecutorError>>(2);
        let cancelled = Arc::new(AtomicBool::new(false));

        let local_exec = Arc::clone(&self.local_executor);
        let remote_exec = Arc::clone(&self.remote_executor);
        let task_local = task.clone();
        let task_remote = task.clone();
        let tx_local = tx.clone();
        let tx_remote = tx;
        let cancelled_local = Arc::clone(&cancelled);
        let cancelled_remote = Arc::clone(&cancelled);
        let grace = self.remote_grace_period;

        std::thread::spawn(move || {
            let res = local_exec.execute(&task_local);
            if !cancelled_local.load(Ordering::Relaxed) {
                let _ = tx_local.send(res);
            }
        });

        std::thread::spawn(move || {
            if !grace.is_zero() {
                std::thread::sleep(grace);
            }
            if !cancelled_remote.load(Ordering::Relaxed) {
                let res = remote_exec.execute(&task_remote);
                if !cancelled_remote.load(Ordering::Relaxed) {
                    let _ = tx_remote.send(res);
                }
            }
        });

        match rx.recv() {
            Ok(outcome_res) => {
                cancelled.store(true, Ordering::SeqCst);
                outcome_res
            }
            Err(_) => Err(ExecutorError::Spawn {
                command: task.label.clone(),
                source: std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "Dynamic racing executors terminated without outcome",
                ),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fish_executor::{CommandSpec, TaskStatus};
    use std::time::Instant;

    struct FastMockExecutor;
    impl TaskExecutor for FastMockExecutor {
        fn execute(&self, _task: &Task) -> Result<TaskOutcome, ExecutorError> {
            Ok(TaskOutcome {
                status: TaskStatus::Executed,
                exit_code: Some(0),
                duration: Duration::from_millis(5),
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    struct SlowMockExecutor;
    impl TaskExecutor for SlowMockExecutor {
        fn execute(&self, _task: &Task) -> Result<TaskOutcome, ExecutorError> {
            std::thread::sleep(Duration::from_millis(200));
            Ok(TaskOutcome {
                status: TaskStatus::Executed,
                exit_code: Some(0),
                duration: Duration::from_millis(200),
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    #[test]
    fn test_dynamic_racing_fast_local_wins() {
        let racer = DynamicRacingExecutor::new(
            FastMockExecutor,
            SlowMockExecutor,
            Duration::from_millis(10),
        );
        let spec = CommandSpec::new("echo");
        let task = Task::new("test", "echo", spec);
        let start = Instant::now();
        let outcome = racer.execute_race(&task).unwrap();
        assert_eq!(outcome.status, TaskStatus::Executed);
        assert!(start.elapsed() < Duration::from_millis(150));
    }
}
