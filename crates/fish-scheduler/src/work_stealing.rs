#![forbid(unsafe_code)]

use fish_executor::{Task, TaskExecutor, TaskOutcome, TaskStatus};
use fish_graph::{BuildGraph, NodeId, TaskState};
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{BuildSummary, FailureRecord, SchedulerError, TaskTiming};

#[derive(Default, Clone)]
pub struct ExecutionHeuristics {
    pub historical_durations: dashmap::DashMap<String, Duration>,
}

impl ExecutionHeuristics {
    pub fn get_estimated_weight(&self, task: &Task) -> u64 {
        if let Some(avg_duration) = self.historical_durations.get(&task.label) {
            avg_duration.as_millis() as u64
        } else {
            100
        }
    }

    pub fn record_execution(&self, task: &Task, duration: Duration) {
        if let Some(mut existing) = self.historical_durations.get_mut(&task.label) {
            let new_avg = (*existing * 8 + duration * 2) / 10;
            *existing = new_avg;
        } else {
            self.historical_durations
                .insert(task.label.clone(), duration);
        }
    }
}

pub struct WorkStealingScheduler {
    worker_count: usize,
    graph: BuildGraph<Task>,
    executor: Arc<dyn TaskExecutor>,
    heuristics: Arc<ExecutionHeuristics>,
}

impl WorkStealingScheduler {
    pub fn new(
        worker_count: usize,
        graph: BuildGraph<Task>,
        executor: Arc<dyn TaskExecutor>,
    ) -> Self {
        let worker_count = worker_count.max(1);

        Self {
            worker_count,
            graph,
            executor,
            heuristics: Arc::new(ExecutionHeuristics::default()),
        }
    }

    pub fn with_heuristics(mut self, heuristics: Arc<ExecutionHeuristics>) -> Self {
        self.heuristics = heuristics;
        self
    }

    /// Priority score used to order ready tasks: longest tail first, then
    /// estimated weight. Ties are resolved deterministically by `NodeId`.
    fn priority_score(&self, id: NodeId) -> u64 {
        let tail_length = self.compute_tail_length(id) as u64;
        let task = &self.graph.node(id).expect("ready nodes exist").payload;
        let weight = self.heuristics.get_estimated_weight(task);
        tail_length * 1000 + weight
    }

    fn compute_tail_length(&self, node_id: NodeId) -> usize {
        let mut max_depth = 0;
        let mut stack = vec![(node_id, 0)];
        let mut visited = std::collections::HashSet::new();

        while let Some((id, depth)) = stack.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.insert(id);

            max_depth = max_depth.max(depth);

            if let Ok(dependents) = self.graph.dependents(id) {
                for dep in dependents {
                    stack.push((*dep, depth + 1));
                }
            }
        }

        max_depth
    }

    pub fn run(&mut self) -> Result<BuildSummary, SchedulerError> {
        let start = Instant::now();

        self.graph.validate()?;
        self.graph.reset_states();

        let (task_tx, task_rx) = crossbeam_channel::unbounded::<(NodeId, Task)>();
        let (done_tx, done_rx) = crossbeam_channel::unbounded::<(NodeId, TaskOutcome)>();

        // Worker threads pull tasks off the shared queue and publish outcomes.
        // The graph itself stays on the scheduler thread, so task payloads are
        // cloned into the queue when they are dispatched.
        let mut workers = Vec::with_capacity(self.worker_count);
        for _ in 0..self.worker_count {
            let task_rx = task_rx.clone();
            let done_tx = done_tx.clone();
            let executor = Arc::clone(&self.executor);
            let heuristics = Arc::clone(&self.heuristics);
            workers.push(std::thread::spawn(move || {
                while let Ok((id, task)) = task_rx.recv() {
                    let task_start = Instant::now();
                    let outcome = match std::panic::catch_unwind(AssertUnwindSafe(|| {
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
                            TaskOutcome::failed(&task, format!("executor panicked: {message}"))
                        }
                    };
                    heuristics.record_execution(&task, task_start.elapsed());
                    let _ = done_tx.send((id, outcome));
                }
            }));
        }

        let mut in_flight: usize = 0;
        let mut failures: Vec<FailureRecord> = Vec::new();
        let mut timings: Vec<TaskTiming> = Vec::new();

        loop {
            // Dispatch every task whose dependencies have succeeded. `ready`
            // only returns `Pending` nodes, so marking them `Running` prevents
            // double dispatch.
            let mut ready = self.graph.ready_nodes();
            ready.sort_by_key(|id| (std::cmp::Reverse(self.priority_score(*id)), id.index()));
            for id in ready {
                let task = self
                    .graph
                    .node(id)
                    .expect("ready nodes exist")
                    .payload
                    .clone();
                self.graph.set_state(id, TaskState::Running)?;
                task_tx.send((id, task)).expect("workers are alive");
                in_flight += 1;
            }

            if in_flight == 0 {
                break;
            }

            // Wait for at least one completion, then drain any that arrived
            // concurrently before dispatching the newly unblocked tasks.
            let (id, outcome) = done_rx.recv().map_err(|_| SchedulerError::Stalled)?;
            in_flight -= 1;
            self.apply_outcome(id, outcome, &mut failures, &mut timings)?;
            while let Ok((id, outcome)) = done_rx.try_recv() {
                in_flight -= 1;
                self.apply_outcome(id, outcome, &mut failures, &mut timings)?;
            }
        }

        // Close the queue so workers observe the channel shutdown and exit.
        drop(task_tx);
        for worker in workers {
            let _ = worker.join();
        }

        Ok(BuildSummary::from_graph(
            &self.graph,
            start.elapsed(),
            self.worker_count,
            failures,
            timings,
        ))
    }

    fn apply_outcome(
        &mut self,
        id: NodeId,
        outcome: TaskOutcome,
        failures: &mut Vec<FailureRecord>,
        timings: &mut Vec<TaskTiming>,
    ) -> Result<(), SchedulerError> {
        // A dependent that was already cancelled by a failure cascade must not
        // be re-marked as Succeeded when its own worker finally reports back.
        if self.graph.state(id)? == TaskState::Cancelled {
            return Ok(());
        }

        let task = self.graph.node(id).map(|node| node.payload.clone());
        if let Some(task) = &task {
            timings.push(TaskTiming {
                label: task.label.clone(),
                description: task.description.clone(),
                start_offset: Duration::ZERO,
                duration: outcome.duration,
                node_id: id,
                worker_id: 0,
                status: outcome.status,
            });
        }

        match outcome.status {
            TaskStatus::Executed => self.graph.set_state(id, TaskState::Succeeded)?,
            TaskStatus::Cached => self.graph.set_state(id, TaskState::Cached)?,
            TaskStatus::Failed => {
                self.graph.mark_failed(id)?;
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fish_executor::{CommandSpec, ExecutorError};
    use std::collections::HashSet;
    use std::sync::Arc;

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

    struct SelectiveExecutor {
        fail: HashSet<String>,
    }

    impl TaskExecutor for SelectiveExecutor {
        fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
            if self.fail.contains(&task.label) {
                Ok(TaskOutcome::failed(task, "boom"))
            } else {
                Ok(TaskOutcome::executed(task))
            }
        }
    }

    #[test]
    fn test_ml_work_stealing_distribution() {
        let mut graph = fish_graph::BuildGraph::new();

        for i in 0..10 {
            let spec = CommandSpec::new("echo").arg(format!("task_{}", i));
            let task = Task::new(format!("task_{}", i), spec.command_line(), spec);
            graph.add_node(task);
        }

        let executor = Arc::new(SelectiveExecutor {
            fail: HashSet::new(),
        });
        let mut scheduler = WorkStealingScheduler::new(4, graph, executor);

        let result = scheduler.run();
        assert!(result.is_ok());

        let summary = result.unwrap();
        assert_eq!(summary.total, 10);
        assert_eq!(summary.executed, 10);
    }

    #[test]
    fn work_stealing_runs_transitive_dependencies() {
        let graph = chain_graph(&["a", "b", "c"]);
        let executor = Arc::new(SelectiveExecutor {
            fail: HashSet::new(),
        });
        let mut scheduler = WorkStealingScheduler::new(2, graph, executor);

        let summary = scheduler.run().unwrap();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.executed, 3);
        assert_eq!(summary.failed, 0);
    }

    #[test]
    fn work_stealing_propagates_failures() {
        // a -> b -> c with b failing: b fails, c is cancelled, a succeeds.
        let graph = chain_graph(&["a", "b", "c"]);
        let executor = Arc::new(SelectiveExecutor {
            fail: HashSet::from(["b".to_string()]),
        });
        let mut scheduler = WorkStealingScheduler::new(2, graph, executor);

        let summary = scheduler.run().unwrap();
        assert_eq!(summary.executed, 1, "only `a` completes");
        assert_eq!(summary.failed, 1, "`b` fails");
        assert_eq!(summary.cancelled, 1, "`c` is cancelled");
        assert_eq!(summary.failures.len(), 1);
    }
}
