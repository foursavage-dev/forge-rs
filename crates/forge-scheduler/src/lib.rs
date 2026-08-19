#![forbid(unsafe_code)]

pub mod bin_packing;
pub mod jobserver_pool;
pub mod racing;
pub mod work_stealing;

pub use bin_packing::{AgentBucket, DteBinPacker, TaskTimingEstimate};
pub use jobserver_pool::JobserverPool;
pub use racing::DynamicRacingExecutor;

use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use forge_executor::{Task, TaskExecutor, TaskOutcome, TaskStatus};
use forge_graph::{BuildGraph, GraphError, NodeId, TaskState};

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("invalid build graph: {0}")]
    InvalidGraph(#[from] GraphError),

    #[error("scheduler stalled with pending tasks; this is a bug, please report it")]
    Stalled,
}

#[derive(Debug, Clone)]
pub struct TaskTiming {
    pub label: String,
    pub description: String,
    pub start_offset: Duration,
    pub duration: Duration,
    pub node_id: NodeId,
    pub worker_id: usize,
    pub status: TaskStatus,
}

impl TaskTiming {
    pub fn new(label: impl Into<String>, duration: Duration, node_id: NodeId) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            description: label,
            start_offset: Duration::ZERO,
            duration,
            node_id,
            worker_id: 0,
            status: TaskStatus::Executed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FailureRecord {
    pub label: String,
    pub description: String,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct BuildSummary {
    pub total: usize,
    pub executed: usize,
    pub cached: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub duration: Duration,
    pub workers: usize,
    pub failures: Vec<FailureRecord>,
    pub timings: Vec<TaskTiming>,
}

impl BuildSummary {
    fn from_graph(
        graph: &BuildGraph<Task>,
        duration: Duration,
        workers: usize,
        failures: Vec<FailureRecord>,
        timings: Vec<TaskTiming>,
    ) -> Self {
        let mut executed = 0;
        let mut cached = 0;
        let mut failed = 0;
        let mut cancelled = 0;
        for node in graph.nodes() {
            match node.state {
                TaskState::Succeeded => executed += 1,
                TaskState::Cached => cached += 1,
                TaskState::Failed => failed += 1,
                TaskState::Cancelled => cancelled += 1,
                TaskState::Pending | TaskState::Running | TaskState::Ready | TaskState::Skipped => {
                }
            }
        }
        Self {
            total: graph.len(),
            executed,
            cached,
            failed,
            cancelled,
            duration,
            workers,
            failures,
            timings,
        }
    }

    pub fn succeeded(&self) -> bool {
        self.failed == 0
    }

    pub fn to_chrome_trace(&self) -> serde_json::Value {
        let mut events = Vec::new();

        events.push(serde_json::json!({
            "name": "process_name",
            "ph": "M",
            "pid": 1,
            "args": {
                "name": "Forge Build Engine"
            }
        }));

        for w in 0..self.workers {
            events.push(serde_json::json!({
                "name": "thread_name",
                "ph": "M",
                "pid": 1,
                "tid": w,
                "args": {
                    "name": format!("Worker {w}")
                }
            }));
        }

        for timing in &self.timings {
            let status_str = match timing.status {
                TaskStatus::Executed => "executed",
                TaskStatus::Cached => "cached",
                TaskStatus::Failed => "failed",
            };
            events.push(serde_json::json!({
                "name": timing.label,
                "cat": "task",
                "ph": "X",
                "ts": timing.start_offset.as_micros(),
                "dur": timing.duration.as_micros(),
                "pid": 1,
                "tid": timing.worker_id,
                "args": {
                    "description": timing.description,
                    "status": status_str,
                    "duration_ms": timing.duration.as_secs_f64() * 1000.0,
                }
            }));
        }

        serde_json::json!({
            "traceEvents": events,
            "displayTimeUnit": "ms",
            "otherData": {
                "forge_version": env!("CARGO_PKG_VERSION"),
                "total_duration_ms": self.duration.as_secs_f64() * 1000.0,
                "total_tasks": self.total,
                "executed": self.executed,
                "cached": self.cached,
                "failed": self.failed,
            }
        })
    }

    pub fn write_chrome_trace(&self, path: &std::path::Path) -> std::io::Result<()> {
        let trace = self.to_chrome_trace();
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &trace)?;
        Ok(())
    }

    pub fn critical_path(&self, graph: &BuildGraph<Task>) -> (Duration, Vec<String>) {
        let timing_map: HashMap<NodeId, Duration> = self
            .timings
            .iter()
            .map(|t| (t.node_id, t.duration))
            .collect();

        let topo = graph.topological_order();
        let mut longest_to: HashMap<NodeId, Duration> = HashMap::new();
        let mut predecessor: HashMap<NodeId, Option<NodeId>> = HashMap::new();

        for &id in &topo {
            let own = timing_map.get(&id).copied().unwrap_or(Duration::ZERO);
            let (best_pred, best_cost) = graph
                .deps(id)
                .unwrap_or_default()
                .iter()
                .filter_map(|dep| longest_to.get(dep).map(|cost| (Some(*dep), *cost)))
                .max_by_key(|(_, cost)| *cost)
                .unwrap_or((None, Duration::ZERO));
            longest_to.insert(id, best_cost + own);
            predecessor.insert(id, best_pred);
        }

        let (&end, &total) = longest_to
            .iter()
            .max_by_key(|(_, cost)| *cost)
            .unwrap_or((&NodeId::from(0), &Duration::ZERO));

        let mut path = Vec::new();
        let mut current = Some(end);
        while let Some(id) = current {
            if let Some(node) = graph.node(id) {
                path.push(node.payload.label.clone());
            }
            current = predecessor.get(&id).copied().flatten();
        }
        path.reverse();
        (total, path)
    }
}

type Outcome = (NodeId, TaskOutcome, usize, Duration);

#[allow(clippy::too_many_arguments)]
fn process_completion<G>(
    graph: &mut BuildGraph<Task>,
    in_flight: &mut usize,
    failures: &mut Vec<FailureRecord>,
    timings: &mut Vec<TaskTiming>,
    on_progress: &mut G,
    id: NodeId,
    outcome: TaskOutcome,
    worker_id: usize,
    start_offset: Duration,
) -> Result<(), SchedulerError>
where
    G: FnMut(&Task, &TaskOutcome),
{
    *in_flight -= 1;
    let task = graph.node(id).map(|node| node.payload.clone());
    if let Some(task) = &task {
        timings.push(TaskTiming {
            label: task.label.clone(),
            description: task.description.clone(),
            start_offset,
            duration: outcome.duration,
            node_id: id,
            worker_id,
            status: outcome.status,
        });
    }
    match outcome.status {
        TaskStatus::Executed => graph.set_state(id, TaskState::Succeeded)?,
        TaskStatus::Cached => graph.set_state(id, TaskState::Cached)?,
        TaskStatus::Failed => {
            graph.mark_failed(id)?;
            if let Some(task) = &task {
                failures.push(FailureRecord {
                    label: task.label.clone(),
                    description: task.description.clone(),
                    stdout: outcome.stdout.clone(),
                    stderr: outcome.stderr.clone(),
                });
            }
        }
    }
    if let Some(task) = &task {
        on_progress(task, &outcome);
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Scheduler {
    workers: usize,
    critical_path: bool,
    ram_threshold: Option<u8>,
    ram_floor: usize,
}

/// Returns the number of workers that may run concurrently once the system's
/// free memory falls below `threshold_percent` of the total: the build is
/// throttled down to `floor` workers so compilers do not get OOM-killed.
pub fn ram_capped_workers(
    available: u64,
    total: u64,
    requested: usize,
    floor: usize,
    threshold_percent: u8,
) -> usize {
    let threshold = threshold_percent.min(100) as u64;
    let constrained = total > 0 && available.saturating_mul(100) < total.saturating_mul(threshold);
    if constrained {
        floor.max(1).min(requested)
    } else {
        requested
    }
}

impl Scheduler {
    pub fn new(workers: usize) -> Self {
        Self {
            workers: workers.max(1),
            critical_path: true,
            ram_threshold: None,
            ram_floor: 1,
        }
    }

    /// Enables (default) or disables critical-path-first task selection. When
    /// enabled, ready tasks whose dependency tail is the longest are picked
    /// first, which minimizes idle worker gaps on wide build graphs.
    pub fn with_critical_path_priority(mut self, enabled: bool) -> Self {
        self.critical_path = enabled;
        self
    }

    /// Throttles the build to `floor` concurrent workers whenever the system's
    /// available memory drops below `threshold_percent` of the total memory.
    pub fn with_ram_backpressure(mut self, threshold_percent: u8, floor: usize) -> Self {
        self.ram_threshold = Some(threshold_percent.min(100));
        self.ram_floor = floor.max(1);
        self
    }

    pub fn run<E, G>(
        &self,
        graph: &mut BuildGraph<Task>,
        executor: &E,
        mut on_progress: G,
    ) -> Result<BuildSummary, SchedulerError>
    where
        E: TaskExecutor,
        G: FnMut(&Task, &TaskOutcome),
    {
        graph.validate()?;
        graph.reset_states();
        let tail = critical_path_tails(graph);
        let start = Instant::now();
        let (tx, rx): (mpsc::Sender<Outcome>, mpsc::Receiver<Outcome>) = mpsc::channel();
        let mut in_flight: usize = 0;
        let mut failures: Vec<FailureRecord> = Vec::new();
        let mut timings: Vec<TaskTiming> = Vec::new();
        let mut free_workers: Vec<usize> = (0..self.workers).rev().collect();
        let mut last_ram_check = Instant::now() - Duration::from_secs(1);
        let mut ram_limited = false;

        let result = thread::scope(|scope| -> Result<(), SchedulerError> {
            let (task_tx, task_rx) =
                crossbeam_channel::unbounded::<(NodeId, Task, usize, Duration)>();
            for _ in 0..self.workers {
                let task_rx = task_rx.clone();
                let tx = tx.clone();
                scope.spawn(move || {
                    while let Ok((id, task, worker_id, task_start_offset)) = task_rx.recv() {
                        let outcome =
                            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                executor.execute(&task)
                            })) {
                                Ok(Ok(outcome)) => outcome,
                                Ok(Err(error)) => TaskOutcome::failed(&task, error.to_string()),
                                Err(panic) => {
                                    let message = panic
                                        .downcast_ref::<&str>()
                                        .map(|s| s.to_string())
                                        .or_else(|| panic.downcast_ref::<String>().cloned())
                                        .unwrap_or_else(|| "unknown panic".to_string());
                                    TaskOutcome::failed(
                                        &task,
                                        format!("executor panicked: {message}"),
                                    )
                                }
                            };
                        let _ = tx.send((id, outcome, worker_id, task_start_offset));
                    }
                });
            }

            let mut ready: Vec<NodeId> = Vec::new();
            let mut ready_index: usize = 0;
            loop {
                if ready_index >= ready.len() {
                    ready = graph.ready_nodes();
                    if self.critical_path {
                        ready.sort_by_key(|id| Reverse(tail[id.index()]));
                    }
                    ready_index = 0;
                }
                let effective_workers = if let Some(threshold) = self.ram_threshold {
                    if last_ram_check.elapsed() >= Duration::from_millis(500) {
                        last_ram_check = Instant::now();
                        let mut sys = sysinfo::System::new_all();
                        sys.refresh_memory();
                        ram_limited = sys.available_memory().saturating_mul(100)
                            < sys.total_memory().saturating_mul(threshold as u64);
                    }
                    if ram_limited {
                        self.ram_floor.min(self.workers)
                    } else {
                        self.workers
                    }
                } else {
                    self.workers
                };
                while in_flight < effective_workers && ready_index < ready.len() {
                    let id = ready[ready_index];
                    ready_index += 1;
                    graph.set_state(id, TaskState::Running)?;
                    let task = graph.node(id).expect("ready nodes exist").payload.clone();
                    in_flight += 1;
                    let worker_id = free_workers.pop().unwrap_or(0);
                    let task_start_offset = start.elapsed();
                    let _ = task_tx.send((id, task, worker_id, task_start_offset));
                }

                if graph.nodes().iter().all(|node| node.state.is_terminal()) {
                    break;
                }

                if in_flight == 0 {
                    return Err(SchedulerError::Stalled);
                }

                let (id, outcome, worker_id, task_start_offset) =
                    rx.recv().map_err(|_| SchedulerError::Stalled)?;
                free_workers.push(worker_id);
                process_completion(
                    graph,
                    &mut in_flight,
                    &mut failures,
                    &mut timings,
                    &mut on_progress,
                    id,
                    outcome,
                    worker_id,
                    task_start_offset,
                )?;
                while let Ok((id, outcome, worker_id, task_start_offset)) = rx.try_recv() {
                    free_workers.push(worker_id);
                    process_completion(
                        graph,
                        &mut in_flight,
                        &mut failures,
                        &mut timings,
                        &mut on_progress,
                        id,
                        outcome,
                        worker_id,
                        task_start_offset,
                    )?;
                }
            }
            Ok(())
        });

        result?;
        Ok(BuildSummary::from_graph(
            graph,
            start.elapsed(),
            self.workers,
            failures,
            timings,
        ))
    }
}

/// Longest path (in edges) from each node down to a leaf of the dependency
/// DAG, measured through `dependents` (reverse topological order); tasks with
/// heavier tails are scheduled first.
fn critical_path_tails(graph: &BuildGraph<Task>) -> Vec<usize> {
    let mut tail = vec![0usize; graph.len()];
    for id in graph.topological_order().into_iter().rev() {
        let longest_dependent = graph
            .dependents(id)
            .unwrap_or_default()
            .iter()
            .map(|dep| tail[dep.index()])
            .max()
            .unwrap_or(0);
        tail[id.index()] = longest_dependent + 1;
    }
    tail
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_executor::{CommandSpec, ExecutorError, ProcessExecutor, Task};
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn chain_graph(labels: &[&str]) -> BuildGraph<Task> {
        let mut graph = BuildGraph::new();
        let mut ids = Vec::new();
        for label in labels {
            let spec = CommandSpec::new("echo").arg(*label);
            ids.push(graph.add_node(Task::new(String::from(*label), spec.command_line(), spec)));
        }
        for window in ids.windows(2) {
            graph
                .add_dependency(window[0], window[1])
                .expect("chain edges are acyclic");
        }
        graph
    }

    fn fanout_graph(labels: &[&str]) -> BuildGraph<Task> {
        let mut graph = BuildGraph::new();
        let mut ids = Vec::new();
        for label in labels {
            let spec = CommandSpec::new("echo").arg(*label);
            ids.push(graph.add_node(Task::new(String::from(*label), spec.command_line(), spec)));
        }
        for id in ids.iter().skip(1) {
            graph
                .add_dependency(ids[0], *id)
                .expect("fanout edges are acyclic");
        }
        graph
    }

    #[test]
    fn critical_path_reports_the_longest_dependency_chain() {
        let mut graph = BuildGraph::new();
        let a = graph.add_node(Task::new(
            "a".to_string(),
            "a".to_string(),
            CommandSpec::new("echo").arg("a"),
        ));
        let b = graph.add_node(Task::new(
            "b".to_string(),
            "b".to_string(),
            CommandSpec::new("echo").arg("b"),
        ));
        let c = graph.add_node(Task::new(
            "c".to_string(),
            "c".to_string(),
            CommandSpec::new("echo").arg("c"),
        ));
        graph.add_dependency(a, b).expect("edge a -> b");
        graph.add_dependency(a, c).expect("edge a -> c");
        graph.add_dependency(b, c).expect("edge b -> c");

        let timings = vec![
            TaskTiming::new("a", Duration::from_secs(1), a),
            TaskTiming::new("b", Duration::from_secs(3), b),
            TaskTiming::new("c", Duration::from_secs(1), c),
        ];
        let summary = BuildSummary::from_graph(&graph, Duration::ZERO, 2, vec![], timings);

        let (total, path) = summary.critical_path(&graph);
        assert_eq!(
            total,
            Duration::from_secs(5),
            "a + b + c dominates the diamond"
        );
        assert_eq!(path, vec!["a", "b", "c"]);
    }

    #[derive(Default)]
    struct FakeExecutor {
        order: Mutex<Vec<String>>,
        max_concurrent: AtomicUsize,
        concurrent: AtomicUsize,
        delays: HashMap<String, Duration>,
        fail: Vec<String>,
        cached: Vec<String>,
    }

    impl FakeExecutor {
        fn with_delay(mut self, label: &str, delay: Duration) -> Self {
            self.delays.insert(label.to_string(), delay);
            self
        }

        fn failing(mut self, labels: &[&str]) -> Self {
            self.fail = labels.iter().map(|s| s.to_string()).collect();
            self
        }
    }

    impl TaskExecutor for FakeExecutor {
        fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
            if self.cached.contains(&task.label) {
                return Ok(TaskOutcome::cached(task));
            }
            let current = self.concurrent.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_concurrent.fetch_max(current, Ordering::SeqCst);
            if let Some(delay) = self.delays.get(&task.label) {
                thread::sleep(*delay);
            }
            let outcome = if self.fail.contains(&task.label) {
                TaskOutcome::failed(task, "boom")
            } else {
                TaskOutcome::executed(task)
            };
            self.order.lock().unwrap().push(task.label.clone());
            self.concurrent.fetch_sub(1, Ordering::SeqCst);
            Ok(outcome)
        }
    }

    #[test]
    fn executes_a_chain_in_order() {
        let mut graph = chain_graph(&["a", "b", "c"]);
        let executor = FakeExecutor::default();
        let scheduler = Scheduler::new(4);
        let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.executed, 3);
        assert_eq!(summary.failed, 0);
        assert!(summary.succeeded());
        assert_eq!(executor.order.lock().unwrap().as_slice(), ["a", "b", "c"]);
    }

    #[test]
    fn runs_independent_tasks_in_parallel() {
        let mut graph = fanout_graph(&["root", "leaf1", "leaf2"]);
        let executor = FakeExecutor::default()
            .with_delay("leaf1", Duration::from_millis(50))
            .with_delay("leaf2", Duration::from_millis(50));
        let scheduler = Scheduler::new(4);
        let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();
        assert_eq!(summary.executed, 3);
        assert!(summary.succeeded());

        assert_eq!(executor.max_concurrent.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn respects_the_worker_limit() {
        let mut graph = fanout_graph(&["root", "l1", "l2", "l3", "l4"]);
        let executor = FakeExecutor::default()
            .with_delay("l1", Duration::from_millis(20))
            .with_delay("l2", Duration::from_millis(20))
            .with_delay("l3", Duration::from_millis(20))
            .with_delay("l4", Duration::from_millis(20));
        let scheduler = Scheduler::new(2);
        let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();
        assert_eq!(summary.executed, 5);
        assert_eq!(executor.max_concurrent.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn a_failure_cancels_dependents() {
        let mut graph = chain_graph(&["a", "b", "c"]);
        let executor = FakeExecutor::default().failing(&["b"]);
        let scheduler = Scheduler::new(2);
        let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();
        assert_eq!(summary.executed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.cancelled, 1);
        assert!(!summary.succeeded());
        assert_eq!(summary.failures.len(), 1);
        assert_eq!(summary.failures[0].label, "b");
        assert_eq!(executor.order.lock().unwrap().as_slice(), ["a", "b"]);
    }

    #[test]
    fn cached_tasks_are_counted() {
        let mut graph = chain_graph(&["a", "b"]);
        let executor = FakeExecutor {
            cached: vec!["b".to_string()],
            ..FakeExecutor::default()
        };
        let scheduler = Scheduler::new(2);
        let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();
        assert_eq!(summary.cached, 1);
        assert_eq!(summary.executed, 1);
        assert!(summary.succeeded());
        assert_eq!(executor.order.lock().unwrap().as_slice(), ["a"]);
    }

    #[test]
    fn reports_executor_spawn_errors_as_failures() {
        let executor = ProcessExecutor::new(false);
        let spec = CommandSpec::new("forge-definitely-not-a-program-xyz");
        let task = Task::new("missing-program", spec.command_line(), spec);
        let mut graph = BuildGraph::new();
        graph.add_node(task);
        let scheduler = Scheduler::new(1);
        let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();
        assert_eq!(summary.failed, 1);
        assert!(summary.failures[0].stderr.contains("failed to spawn"));
    }

    #[test]
    fn reports_failed_command_exit_codes() {
        let executor = ProcessExecutor::new(false);
        let spec = CommandSpec::new("cargo")
            .arg("metadata")
            .arg("--manifest-path")
            .arg("definitely-missing-path/Cargo.toml");
        let task = Task::new("failing-cargo", spec.command_line(), spec);
        let mut graph = BuildGraph::new();
        graph.add_node(task);
        let scheduler = Scheduler::new(1);
        let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.failures[0].label, "failing-cargo");
        assert!(!summary.failures[0].stderr.is_empty());
    }

    #[test]
    fn worker_count_defaults_to_at_least_one() {
        assert_eq!(Scheduler::new(0).workers, 1);
        assert_eq!(Scheduler::new(1).workers, 1);
        assert_eq!(Scheduler::new(4).workers, 4);
    }

    #[test]
    fn critical_path_priority_schedules_heavy_chains_first() {
        let mut graph = BuildGraph::new();
        let _b = graph.add_node(Task::new(
            "b".to_string(),
            "b".to_string(),
            CommandSpec::new("echo").arg("b"),
        ));
        let a0 = graph.add_node(Task::new(
            "a0".to_string(),
            "a0".to_string(),
            CommandSpec::new("echo").arg("a0"),
        ));
        let a1 = graph.add_node(Task::new(
            "a1".to_string(),
            "a1".to_string(),
            CommandSpec::new("echo").arg("a1"),
        ));
        graph.add_dependency(a0, a1).expect("a0 -> a1");

        let executor = FakeExecutor::default();
        let scheduler = Scheduler::new(1);
        let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();
        assert!(summary.succeeded());

        let order = executor.order.lock().unwrap().clone();
        assert_eq!(
            order.first(),
            Some(&"a0".to_string()),
            "the node whose tail is longer (a0 -> a1) must be picked before the independent b"
        );
        let a0_pos = order.iter().position(|l| l == "a0").unwrap();
        let a1_pos = order.iter().position(|l| l == "a1").unwrap();
        assert!(a0_pos < a1_pos, "a1 cannot run before its dependency a0");
    }

    #[test]
    fn without_priority_the_node_order_wins() {
        let mut graph = BuildGraph::new();
        let _b = graph.add_node(Task::new(
            "b".to_string(),
            "b".to_string(),
            CommandSpec::new("echo").arg("b"),
        ));
        let a0 = graph.add_node(Task::new(
            "a0".to_string(),
            "a0".to_string(),
            CommandSpec::new("echo").arg("a0"),
        ));
        let a1 = graph.add_node(Task::new(
            "a1".to_string(),
            "a1".to_string(),
            CommandSpec::new("echo").arg("a1"),
        ));
        graph.add_dependency(a0, a1).expect("a0 -> a1");

        let executor = FakeExecutor::default();
        let scheduler = Scheduler::new(1).with_critical_path_priority(false);
        scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();

        let order = executor.order.lock().unwrap().clone();
        assert_eq!(order, vec!["b", "a0", "a1"]);
    }

    #[test]
    fn ram_capped_workers_only_throttles_below_the_threshold() {
        assert_eq!(ram_capped_workers(8000, 10000, 8, 1, 20), 8);
        assert_eq!(ram_capped_workers(1000, 10000, 8, 1, 20), 1);
        assert_eq!(ram_capped_workers(0, 10000, 8, 2, 10), 2);
        assert_eq!(
            ram_capped_workers(1000, 0, 8, 2, 10),
            8,
            "unknown totals never throttle"
        );
        assert_eq!(
            ram_capped_workers(1, 100, 4, 1, 100),
            1,
            "100% free always throttles"
        );
        assert_eq!(
            ram_capped_workers(1, 100, 4, 4, 100),
            4,
            "floor is capped by requested"
        );
        assert_eq!(ram_capped_workers(99, 100, 4, 2, 100), 2);
    }

    #[test]
    fn ram_backpressure_still_completes_a_build() {
        let mut graph = chain_graph(&["a", "b", "c"]);
        let executor = FakeExecutor::default();
        let scheduler = Scheduler::new(4).with_ram_backpressure(1, 1);
        let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();
        assert!(summary.succeeded());
        assert_eq!(summary.executed, 3);
    }

    #[test]
    fn test_chrome_trace_generation() {
        let mut graph = chain_graph(&["task1", "task2"]);
        let executor = FakeExecutor::default();
        let scheduler = Scheduler::new(2);
        let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();

        let trace = summary.to_chrome_trace();
        let events = trace["traceEvents"].as_array().unwrap();
        assert!(events.iter().any(|e| e["name"] == "process_name"));
        assert!(
            events
                .iter()
                .any(|e| e["name"] == "thread_name" && e["args"]["name"] == "Worker 0")
        );
        assert!(events.iter().any(|e| e["name"] == "task1"));
        assert!(events.iter().any(|e| e["name"] == "task2"));
    }
}
