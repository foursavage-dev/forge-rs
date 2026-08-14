#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::client::RemoteWorkerClient;
use forge_executor::{ExecutorError, ProcessExecutor, Task, TaskExecutor, TaskOutcome};

#[derive(Clone)]
pub struct ClusterExecutor {
    workers: Vec<RemoteWorkerClient>,
    local_executor: Option<Arc<dyn TaskExecutor>>,
    round_robin_idx: Arc<AtomicUsize>,
    failover_to_local: bool,
}

impl ClusterExecutor {
    pub fn new(workers: Vec<RemoteWorkerClient>) -> Self {
        Self {
            workers,
            local_executor: Some(Arc::new(ProcessExecutor::default())),
            round_robin_idx: Arc::new(AtomicUsize::new(0)),
            failover_to_local: true,
        }
    }

    pub fn with_local_fallback(
        workers: Vec<RemoteWorkerClient>,
        local: Arc<dyn TaskExecutor>,
    ) -> Self {
        Self {
            workers,
            local_executor: Some(local),
            round_robin_idx: Arc::new(AtomicUsize::new(0)),
            failover_to_local: true,
        }
    }

    pub fn without_fallback(workers: Vec<RemoteWorkerClient>) -> Self {
        Self {
            workers,
            local_executor: None,
            round_robin_idx: Arc::new(AtomicUsize::new(0)),
            failover_to_local: false,
        }
    }

    /// Route tasks with a snapshot of the source tree attached, so workers on
    /// a different filesystem can still execute them.
    pub fn with_source_packaging(mut self) -> Self {
        self.workers = self
            .workers
            .into_iter()
            .map(|worker| worker.with_source_packaging())
            .collect();
        self
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn healthy_workers(&self) -> Vec<RemoteWorkerClient> {
        self.workers
            .iter()
            .filter(|w| w.ping().is_ok())
            .cloned()
            .collect()
    }
}

impl TaskExecutor for ClusterExecutor {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        if self.workers.is_empty() {
            if let Some(local) = &self.local_executor {
                return local.execute(task);
            } else {
                return Err(ExecutorError::Spawn {
                    command: task.label.clone(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "no remote workers configured and no local fallback",
                    ),
                });
            }
        }

        let total_workers = self.workers.len();
        let start_idx = self.round_robin_idx.fetch_add(1, Ordering::SeqCst) % total_workers;

        for offset in 0..total_workers {
            let idx = (start_idx + offset) % total_workers;
            let worker = &self.workers[idx];

            match worker.execute(task) {
                Ok(outcome) => return Ok(outcome),
                Err(e) => {
                    if offset + 1 == total_workers && !self.failover_to_local {
                        return Err(e);
                    }
                }
            }
        }

        if let Some(local) = &self.local_executor {
            if self.failover_to_local {
                return local.execute(task);
            }
        }

        Err(ExecutorError::Spawn {
            command: task.label.clone(),
            source: std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "all remote workers failed and local fallback unavailable",
            ),
        })
    }
}
