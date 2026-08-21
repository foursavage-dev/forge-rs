#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CriticalPathError {
    /// A dependency cycle was detected while walking downstream edges.
    Cycle(String),
}

impl fmt::Display for CriticalPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CriticalPathError::Cycle(node) => {
                write!(f, "dependency cycle detected at task `{node}`")
            }
        }
    }
}

impl std::error::Error for CriticalPathError {}

#[derive(Debug, Clone, Default)]
pub struct TaskHistoricalProfile {
    pub average_duration_ms: u64,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CriticalPathScheduler {
    profiles: HashMap<String, TaskHistoricalProfile>,
}

impl CriticalPathScheduler {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    pub fn record_task_duration(&mut self, task_name: &str, duration_ms: u64) {
        let entry = self.profiles.entry(task_name.to_string()).or_default();
        let total = entry.average_duration_ms * (entry.sample_count as u64) + duration_ms;
        entry.sample_count += 1;
        entry.average_duration_ms = total / (entry.sample_count as u64);
    }

    pub fn compute_critical_weights(
        &self,
        task_names: &[String],
        adjacency: &HashMap<String, Vec<String>>,
    ) -> Result<HashMap<String, u64>, CriticalPathError> {
        let mut weights = HashMap::new();
        // Memoize each task's longest downstream cost so shared sub-paths
        // (diamonds) are only walked once instead of recomputed exponentially.
        let mut memo: HashMap<String, u64> = HashMap::new();
        let mut visiting: HashSet<String> = HashSet::new();

        for name in task_names {
            let base_cost = self
                .profiles
                .get(name)
                .map(|p| p.average_duration_ms)
                .unwrap_or(100);

            let downstream_cost =
                self.longest_downstream_path(name, adjacency, &mut memo, &mut visiting)?;
            weights.insert(name.clone(), base_cost + downstream_cost);
        }

        Ok(weights)
    }

    fn longest_downstream_path(
        &self,
        node: &str,
        adjacency: &HashMap<String, Vec<String>>,
        memo: &mut HashMap<String, u64>,
        visiting: &mut HashSet<String>,
    ) -> Result<u64, CriticalPathError> {
        if let Some(&cached) = memo.get(node) {
            return Ok(cached);
        }
        if !visiting.insert(node.to_string()) {
            return Err(CriticalPathError::Cycle(node.to_string()));
        }

        let mut max_cost = 0;
        if let Some(neighbors) = adjacency.get(node) {
            for neighbor in neighbors {
                let cost = self
                    .profiles
                    .get(neighbor)
                    .map(|p| p.average_duration_ms)
                    .unwrap_or(100)
                    + self.longest_downstream_path(neighbor, adjacency, memo, visiting)?;
                if cost > max_cost {
                    max_cost = cost;
                }
            }
        }

        visiting.remove(node);
        memo.insert(node.to_string(), max_cost);
        Ok(max_cost)
    }

    pub fn prioritize_ready_tasks(
        &self,
        ready_tasks: &mut [String],
        weights: &HashMap<String, u64>,
    ) {
        ready_tasks.sort_by(|a, b| {
            let w_a = weights.get(a).copied().unwrap_or(0);
            let w_b = weights.get(b).copied().unwrap_or(0);
            w_b.cmp(&w_a)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_critical_path_weight_calculation() {
        let mut scheduler = CriticalPathScheduler::new();
        scheduler.record_task_duration("task_a", 500);
        scheduler.record_task_duration("task_b", 1000);
        scheduler.record_task_duration("task_c", 200);

        let mut adj = HashMap::new();
        adj.insert("task_a".to_string(), vec!["task_b".to_string()]);
        adj.insert("task_b".to_string(), vec!["task_c".to_string()]);

        let tasks = vec![
            "task_a".to_string(),
            "task_b".to_string(),
            "task_c".to_string(),
        ];
        let weights = scheduler.compute_critical_weights(&tasks, &adj).unwrap();

        assert_eq!(*weights.get("task_a").unwrap(), 500 + 1000 + 200);
        assert_eq!(*weights.get("task_b").unwrap(), 1000 + 200);
        assert_eq!(*weights.get("task_c").unwrap(), 200);

        let mut ready = vec![
            "task_c".to_string(),
            "task_a".to_string(),
            "task_b".to_string(),
        ];
        scheduler.prioritize_ready_tasks(&mut ready, &weights);
        assert_eq!(ready, vec!["task_a", "task_b", "task_c"]);
    }

    #[test]
    fn compute_critical_weights_detects_cycles_instead_of_overflowing() {
        let scheduler = CriticalPathScheduler::new();

        let mut two_cycle = HashMap::new();
        two_cycle.insert("a".to_string(), vec!["b".to_string()]);
        two_cycle.insert("b".to_string(), vec!["a".to_string()]);
        assert!(matches!(
            scheduler.compute_critical_weights(&names(&["a", "b"]), &two_cycle),
            Err(CriticalPathError::Cycle(_))
        ));

        let mut self_loop = HashMap::new();
        self_loop.insert("x".to_string(), vec!["x".to_string()]);
        assert!(matches!(
            scheduler.compute_critical_weights(&names(&["x"]), &self_loop),
            Err(CriticalPathError::Cycle(_))
        ));
    }

    #[test]
    fn compute_critical_weights_handles_shared_downstream_paths() {
        let mut scheduler = CriticalPathScheduler::new();
        scheduler.record_task_duration("leaf", 10);
        scheduler.record_task_duration("left", 20);
        scheduler.record_task_duration("right", 30);

        // Diamond: left -> leaf and right -> leaf share the `leaf` subtree.
        let mut adj = HashMap::new();
        adj.insert("left".to_string(), vec!["leaf".to_string()]);
        adj.insert("right".to_string(), vec!["leaf".to_string()]);

        let weights = scheduler
            .compute_critical_weights(&names(&["left", "right", "leaf"]), &adj)
            .unwrap();

        assert_eq!(*weights.get("leaf").unwrap(), 10);
        assert_eq!(*weights.get("left").unwrap(), 20 + 10);
        assert_eq!(*weights.get("right").unwrap(), 30 + 10);
    }

    #[test]
    fn unknown_tasks_get_default_cost_without_panicking() {
        let scheduler = CriticalPathScheduler::new();
        let weights = scheduler
            .compute_critical_weights(&names(&["missing"]), &HashMap::new())
            .unwrap();
        assert_eq!(*weights.get("missing").unwrap(), 100);
    }
}
