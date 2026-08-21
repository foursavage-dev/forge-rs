#![allow(dead_code)]

use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerState {
    Idle,
    Compiling,
    RollingBack,
    Evicted,
}

#[derive(Debug, Clone)]
pub struct DaemonTaskPayload {
    pub task_id: String,
    pub source_content: String,
    pub compiler_flags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DaemonExecutionSummary {
    pub task_id: String,
    pub latency_micros: u128,
    pub exit_code: i32,
    pub memory_reset_ok: bool,
    pub assigned_worker_id: usize,
    pub cache_hit: bool,
}

pub struct PrewarmedCompilerWorker {
    pub worker_id: usize,
    pub state: WorkerState,
    pub generation: u64,
    pub ast_cache: HashMap<String, Vec<u8>>,
    pub total_tasks_processed: usize,
}

pub struct CompilerDaemonPool {
    workers: Arc<Mutex<Vec<PrewarmedCompilerWorker>>>,
    max_cached_entries_per_worker: usize,
}

impl CompilerDaemonPool {
    pub fn new(capacity: usize) -> Self {
        let mut workers = Vec::with_capacity(capacity);
        for id in 0..capacity {
            workers.push(PrewarmedCompilerWorker {
                worker_id: id,
                state: WorkerState::Idle,
                generation: 1,
                ast_cache: HashMap::new(),
                total_tasks_processed: 0,
            });
        }
        Self {
            workers: Arc::new(Mutex::new(workers)),
            max_cached_entries_per_worker: 32,
        }
    }

    pub fn active_worker_count(&self) -> usize {
        self.workers.lock().unwrap().len()
    }

    pub fn dispatch_fast_compile(
        &self,
        payload: DaemonTaskPayload,
    ) -> io::Result<DaemonExecutionSummary> {
        // There is no compiler daemon here; the previous implementation
        // returned a fabricated `exit_code: 0` summary without compiling
        // anything. Fail loudly instead of reporting a successful build.
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "daemon fast-compile is not implemented (task `{}`)",
                payload.task_id
            ),
        ))
    }

    pub fn respawn_dead_workers(&self) -> usize {
        let mut workers = self.workers.lock().unwrap();
        let mut respawned = 0;
        for w in workers.iter_mut() {
            if w.state == WorkerState::Evicted {
                w.state = WorkerState::Idle;
                w.ast_cache.clear();
                w.generation = 1;
                respawned += 1;
            }
        }
        respawned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_daemon_pool_refuses_fake_compile() {
        let pool = CompilerDaemonPool::new(4);
        let payload = DaemonTaskPayload {
            task_id: "module_core_1".to_string(),
            source_content: "pub fn fast_fn() -> u64 { 42 }".to_string(),
            compiler_flags: vec!["-O3".to_string()],
        };

        let result = pool.dispatch_fast_compile(payload);
        assert!(
            result.is_err(),
            "unimplemented daemon compilation must fail loudly"
        );
    }

    #[test]
    fn test_worker_respawn() {
        let pool = CompilerDaemonPool::new(2);
        {
            let mut workers = pool.workers.lock().unwrap();
            workers[0].state = WorkerState::Evicted;
        }

        let count = pool.respawn_dead_workers();
        assert_eq!(count, 1);
        let workers = pool.workers.lock().unwrap();
        assert_eq!(workers[0].state, WorkerState::Idle);
    }
}
