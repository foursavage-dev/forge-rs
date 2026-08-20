#![allow(dead_code)]

use std::collections::HashMap;

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
    ) -> HashMap<String, u64> {
        let mut weights = HashMap::new();

        for name in task_names {
            let base_cost = self
                .profiles
                .get(name)
                .map(|p| p.average_duration_ms)
                .unwrap_or(100);

            let downstream_cost = self.longest_downstream_path(name, adjacency);
            weights.insert(name.clone(), base_cost + downstream_cost);
        }

        weights
    }

    fn longest_downstream_path(&self, node: &str, adjacency: &HashMap<String, Vec<String>>) -> u64 {
        if let Some(neighbors) = adjacency.get(node) {
            let mut max_cost = 0;
            for neighbor in neighbors {
                let cost = self
                    .profiles
                    .get(neighbor)
                    .map(|p| p.average_duration_ms)
                    .unwrap_or(100)
                    + self.longest_downstream_path(neighbor, adjacency);
                if cost > max_cost {
                    max_cost = cost;
                }
            }
            max_cost
        } else {
            0
        }
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
        let weights = scheduler.compute_critical_weights(&tasks, &adj);

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
}
