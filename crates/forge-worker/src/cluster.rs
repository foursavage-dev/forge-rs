#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::client::RemoteWorkerClient;
use forge_executor::{ExecutorError, ProcessExecutor, Task, TaskExecutor, TaskOutcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadBalancingStrategy {
    #[default]
    RoundRobin,
    LeastLoaded,
    Random,
}

#[derive(Debug, Clone)]
pub struct WorkerCircuitBreaker {
    failure_counts: Arc<Mutex<HashMap<String, (usize, Instant)>>>,
    failure_threshold: usize,
    cooldown: Duration,
}

impl WorkerCircuitBreaker {
    pub fn new(failure_threshold: usize, cooldown: Duration) -> Self {
        Self {
            failure_counts: Arc::new(Mutex::new(HashMap::new())),
            failure_threshold,
            cooldown,
        }
    }

    pub fn record_failure(&self, addr: &str) {
        if let Ok(mut map) = self.failure_counts.lock() {
            let entry = map.entry(addr.to_string()).or_insert((0, Instant::now()));
            entry.0 = entry.0.saturating_add(1);
            entry.1 = Instant::now();
        }
    }

    pub fn record_success(&self, addr: &str) {
        if let Ok(mut map) = self.failure_counts.lock() {
            map.remove(addr);
        }
    }

    pub fn is_degraded(&self, addr: &str) -> bool {
        if let Ok(map) = self.failure_counts.lock()
            && let Some((count, last_failure)) = map.get(addr)
            && *count >= self.failure_threshold
            && last_failure.elapsed() < self.cooldown
        {
            return true;
        }
        false
    }

    pub fn failure_count(&self, addr: &str) -> usize {
        if let Ok(map) = self.failure_counts.lock() {
            map.get(addr).map(|(c, _)| *c).unwrap_or(0)
        } else {
            0
        }
    }
}

impl Default for WorkerCircuitBreaker {
    fn default() -> Self {
        Self::new(3, Duration::from_secs(10))
    }
}

#[derive(Clone)]
pub struct ClusterExecutor {
    workers: Vec<RemoteWorkerClient>,
    local_executor: Option<Arc<dyn TaskExecutor>>,
    round_robin_idx: Arc<AtomicUsize>,
    random_seed: Arc<AtomicU64>,
    failover_to_local: bool,
    strategy: LoadBalancingStrategy,
    circuit_breaker: WorkerCircuitBreaker,
}

impl ClusterExecutor {
    pub fn new(workers: Vec<RemoteWorkerClient>) -> Self {
        Self {
            workers,
            local_executor: Some(Arc::new(ProcessExecutor::default())),
            round_robin_idx: Arc::new(AtomicUsize::new(0)),
            random_seed: Arc::new(AtomicU64::new(123456789)),
            failover_to_local: true,
            strategy: LoadBalancingStrategy::RoundRobin,
            circuit_breaker: WorkerCircuitBreaker::default(),
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
            random_seed: Arc::new(AtomicU64::new(123456789)),
            failover_to_local: true,
            strategy: LoadBalancingStrategy::RoundRobin,
            circuit_breaker: WorkerCircuitBreaker::default(),
        }
    }

    pub fn without_fallback(workers: Vec<RemoteWorkerClient>) -> Self {
        Self {
            workers,
            local_executor: None,
            round_robin_idx: Arc::new(AtomicUsize::new(0)),
            random_seed: Arc::new(AtomicU64::new(123456789)),
            failover_to_local: false,
            strategy: LoadBalancingStrategy::RoundRobin,
            circuit_breaker: WorkerCircuitBreaker::default(),
        }
    }

    pub fn with_strategy(mut self, strategy: LoadBalancingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn strategy(&self) -> LoadBalancingStrategy {
        self.strategy
    }

    pub fn with_circuit_breaker(mut self, failure_threshold: usize, cooldown: Duration) -> Self {
        self.circuit_breaker = WorkerCircuitBreaker::new(failure_threshold, cooldown);
        self
    }

    pub fn circuit_breaker(&self) -> &WorkerCircuitBreaker {
        &self.circuit_breaker
    }

    pub fn with_source_packaging(mut self) -> Self {
        self.workers = self
            .workers
            .into_iter()
            .map(|worker| worker.with_source_packaging())
            .collect();
        self
    }

    pub fn with_vfs(mut self, use_vfs: bool) -> Self {
        self.workers = self
            .workers
            .into_iter()
            .map(|worker| worker.with_vfs(use_vfs))
            .collect();
        self
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn healthy_workers(&self) -> Vec<RemoteWorkerClient> {
        self.workers
            .iter()
            .filter(|w| !self.circuit_breaker.is_degraded(&w.server_addr) && w.ping().is_ok())
            .cloned()
            .collect()
    }

    fn next_random_index(&self, max: usize) -> usize {
        if max <= 1 {
            return 0;
        }
        let prev = self
            .random_seed
            .fetch_add(0x9E3779B97F4A7C15, Ordering::Relaxed);
        let mut x = prev ^ (prev >> 30);
        x = x.wrapping_mul(0xBF58476D1CE4E5B9);
        x = x ^ (x >> 27);
        x = x.wrapping_mul(0x94D049BB133111EB);
        x = x ^ (x >> 31);
        (x as usize) % max
    }

    pub fn select_candidate_indices(&self) -> Vec<usize> {
        let total = self.workers.len();
        if total == 0 {
            return Vec::new();
        }

        let mut indices: Vec<usize> = match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let start = self.round_robin_idx.fetch_add(1, Ordering::SeqCst) % total;
                (0..total).map(|offset| (start + offset) % total).collect()
            }
            LoadBalancingStrategy::LeastLoaded => {
                let mut indexed_workers: Vec<(usize, usize, usize)> = self
                    .workers
                    .iter()
                    .enumerate()
                    .map(|(idx, w)| {
                        let failure_cnt = self.circuit_breaker.failure_count(&w.server_addr);
                        let load = match w.ping() {
                            Ok(resp) => resp.health.active_jobs,
                            Err(_) => usize::MAX / 2,
                        };
                        (idx, load, failure_cnt)
                    })
                    .collect();

                indexed_workers.sort_by_key(|&(_, load, failures)| (failures, load));
                indexed_workers.into_iter().map(|(idx, _, _)| idx).collect()
            }
            LoadBalancingStrategy::Random => {
                let start = self.next_random_index(total);
                (0..total).map(|offset| (start + offset) % total).collect()
            }
        };

        indices.sort_by_key(|&idx| {
            let addr = &self.workers[idx].server_addr;
            if self.circuit_breaker.is_degraded(addr) {
                1
            } else {
                0
            }
        });

        indices
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

        let candidates = self.select_candidate_indices();
        let total_candidates = candidates.len();

        for (attempt, &idx) in candidates.iter().enumerate() {
            let worker = &self.workers[idx];

            match worker.execute(task) {
                Ok(outcome) => {
                    self.circuit_breaker.record_success(&worker.server_addr);
                    return Ok(outcome);
                }
                Err(e) => {
                    self.circuit_breaker.record_failure(&worker.server_addr);
                    if attempt + 1 == total_candidates && !self.failover_to_local {
                        return Err(e);
                    }
                }
            }
        }

        if let Some(local) = &self.local_executor
            && self.failover_to_local
        {
            return local.execute(task);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_balancing_strategy_defaults() {
        let workers = vec![
            RemoteWorkerClient::new("127.0.0.1:9001", None),
            RemoteWorkerClient::new("127.0.0.1:9002", None),
        ];
        let cluster = ClusterExecutor::new(workers.clone());
        assert_eq!(cluster.strategy(), LoadBalancingStrategy::RoundRobin);

        let round_robin_order_1 = cluster.select_candidate_indices();
        let round_robin_order_2 = cluster.select_candidate_indices();
        assert_ne!(round_robin_order_1, round_robin_order_2);

        let random_cluster = cluster.with_strategy(LoadBalancingStrategy::Random);
        assert_eq!(random_cluster.strategy(), LoadBalancingStrategy::Random);
        let random_order = random_cluster.select_candidate_indices();
        assert_eq!(random_order.len(), 2);
    }

    #[test]
    fn test_circuit_breaker_degradation() {
        let breaker = WorkerCircuitBreaker::new(2, Duration::from_secs(5));
        let node = "10.0.0.1:8080";

        assert!(!breaker.is_degraded(node));
        assert_eq!(breaker.failure_count(node), 0);

        breaker.record_failure(node);
        assert!(!breaker.is_degraded(node));
        assert_eq!(breaker.failure_count(node), 1);

        breaker.record_failure(node);
        assert!(breaker.is_degraded(node));
        assert_eq!(breaker.failure_count(node), 2);

        breaker.record_success(node);
        assert!(!breaker.is_degraded(node));
        assert_eq!(breaker.failure_count(node), 0);
    }

    #[test]
    fn test_degraded_worker_prioritization() {
        let workers = vec![
            RemoteWorkerClient::new("127.0.0.1:9010", None),
            RemoteWorkerClient::new("127.0.0.1:9011", None),
        ];
        let cluster = ClusterExecutor::new(workers).with_circuit_breaker(1, Duration::from_secs(5));
        cluster.circuit_breaker().record_failure("127.0.0.1:9010");

        let candidates = cluster.select_candidate_indices();
        assert_eq!(candidates[0], 1);
        assert_eq!(candidates[1], 0);
    }
}
