//! `forge-scheduler`: parallel execution of the build graph.
//!
//! Given a validated `BuildGraph<Task>`, the scheduler runs ready tasks on
//! a fixed-size worker pool, propagates failures (cancelling dependents),
//! counts cached tasks, and reports a `BuildSummary`.
//!
//! The scheduler never spawns processes itself: it only talks to the
//! [`TaskExecutor`] trait, so cached wrappers and test doubles plug in
//! seamlessly.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use forge_executor::{Task, TaskExecutor, TaskOutcome, TaskStatus};
use forge_graph::{BuildGraph, GraphError, NodeId, TaskState};

/// Errors surfaced by the scheduler.
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    /// The graph failed structural validation before execution.
    #[error("invalid build graph: {0}")]
    InvalidGraph(#[from] GraphError),
    /// Defensive: the scheduler ran out of work while tasks were still
    /// pending, which indicates a state-machine bug.
    #[error("scheduler stalled with pending tasks; this is a bug, please report it")]
    Stalled,
}

/// A task that failed, and why.
#[derive(Debug, Clone)]
pub struct FailureRecord {
    /// The failing task's label.
    pub label: String,
    /// The command line that failed.
    pub description: String,
    /// Captured stderr of the failed task (or an executor error message).
    pub stderr: String,
}

/// Aggregate results of one `Scheduler::run` invocation.
#[derive(Debug, Clone)]
pub struct BuildSummary {
    /// Total tasks in the graph.
    pub total: usize,
    /// Tasks that actually executed and succeeded.
    pub executed: usize,
    /// Tasks satisfied by the cache without executing.
    pub cached: usize,
    /// Tasks that failed.
    pub failed: usize,
    /// Tasks cancelled because a dependency failed.
    pub cancelled: usize,
    /// Wall-clock time of the whole run.
    pub duration: Duration,
    /// Number of worker threads used.
    pub workers: usize,
    /// Failure details, in order of occurrence.
    pub failures: Vec<FailureRecord>,
}

impl BuildSummary {
    fn from_graph(
        graph: &BuildGraph<Task>,
        duration: Duration,
        workers: usize,
        failures: Vec<FailureRecord>,
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
        }
    }

    /// True when every task succeeded (either by executing or from cache).
    pub fn succeeded(&self) -> bool {
        self.failed == 0
    }
}

// A (node, outcome) pair travelling from worker threads to the scheduler.
type Outcome = (NodeId, TaskOutcome);

/// Apply one finished task to the graph: update the node state, record
/// failures, and notify progress listeners.
fn process_completion<G>(
    graph: &mut BuildGraph<Task>,
    in_flight: &mut usize,
    failures: &mut Vec<FailureRecord>,
    on_progress: &mut G,
    id: NodeId,
    outcome: TaskOutcome,
) -> Result<(), SchedulerError>
where
    G: FnMut(&Task, &TaskOutcome),
{
    *in_flight -= 1;
    let task = graph.node(id).map(|node| node.payload.clone());
    match outcome.status {
        TaskStatus::Executed => graph.set_state(id, TaskState::Succeeded)?,
        TaskStatus::Cached => graph.set_state(id, TaskState::Cached)?,
        TaskStatus::Failed => {
            graph.mark_failed(id)?;
            if let Some(task) = &task {
                failures.push(FailureRecord {
                    label: task.label.clone(),
                    description: task.description.clone(),
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

/// Executes ready tasks in parallel on a fixed worker pool.
#[derive(Debug, Clone)]
pub struct Scheduler {
    workers: usize,
}

impl Scheduler {
    pub fn new(workers: usize) -> Self {
        Self {
            workers: workers.max(1),
        }
    }

    /// Run every task in `graph` to completion.
    ///
    /// Tasks become ready as soon as all their dependencies succeeded or
    /// were cached. When a task fails, its transitive dependents are marked
    /// `Cancelled`. `on_progress` is invoked (on the scheduling thread)
    /// after each task finishes; it is never called from worker threads.
    ///
    /// Worker threads are scoped: `run` returns only after every task has
    /// finished. Panics inside `E::execute` are caught and turned into a
    /// failed outcome so a buggy executor cannot take down a build.
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
        let start = Instant::now();
        let (tx, rx): (mpsc::Sender<Outcome>, mpsc::Receiver<Outcome>) = mpsc::channel();
        let mut in_flight: usize = 0;
        let mut failures: Vec<FailureRecord> = Vec::new();

        let result = thread::scope(|scope| -> Result<(), SchedulerError> {
            let mut ready: Vec<NodeId> = Vec::new();
            let mut ready_index: usize = 0;
            loop {
                // 1. Dispatch all currently ready tasks (up to the worker
                //    limit). `ready` is refreshed lazily only when exhausted.
                if ready_index >= ready.len() {
                    ready = graph.ready_nodes();
                    ready_index = 0;
                }
                while in_flight < self.workers && ready_index < ready.len() {
                    let id = ready[ready_index];
                    ready_index += 1;
                    graph.set_state(id, TaskState::Running)?;
                    let task = graph.node(id).expect("ready nodes exist").payload.clone();
                    in_flight += 1;
                    let tx = tx.clone();
                    scope.spawn(move || {
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
                        let _ = tx.send((id, outcome));
                    });
                }

                // 2. All tasks terminal?
                if graph.nodes().iter().all(|node| node.state.is_terminal()) {
                    break;
                }

                // 3. Nothing in flight (but tasks still pending) can only
                //    happen if a ready task was missed or a state transition
                //    is wrong — better to stop than to wait forever.
                if in_flight == 0 {
                    return Err(SchedulerError::Stalled);
                }

                // 4. Block for at least one completion, then drain whatever
                //    else arrived while we were dispatching.
                let (id, outcome) = rx.recv().map_err(|_| SchedulerError::Stalled)?;
                process_completion(
                    graph,
                    &mut in_flight,
                    &mut failures,
                    &mut on_progress,
                    id,
                    outcome,
                )?;
                while let Ok((id, outcome)) = rx.try_recv() {
                    process_completion(
                        graph,
                        &mut in_flight,
                        &mut failures,
                        &mut on_progress,
                        id,
                        outcome,
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
        ))
    }
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

    /// Deterministic test executor: records execution order, max concurrency,
    /// and can fail or serve-from-cache specific labels.
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
        // root runs alone, then both leaves overlap.
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
}
