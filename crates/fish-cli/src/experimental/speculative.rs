#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct MarkovTransitionModel {
    first_order: HashMap<String, HashMap<String, usize>>,
    second_order: HashMap<(String, String), HashMap<String, usize>>,
    co_occurrence: HashMap<String, HashMap<String, usize>>,
    history: Vec<String>,
}

impl MarkovTransitionModel {
    pub fn new() -> Self {
        Self {
            first_order: HashMap::new(),
            second_order: HashMap::new(),
            co_occurrence: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn record_transition(&mut self, from_file: &str, to_file: &str) {
        let entry = self.first_order.entry(from_file.to_string()).or_default();
        *entry.entry(to_file.to_string()).or_insert(0) += 1;

        if let Some(prev) = self.history.last() {
            let key = (prev.clone(), from_file.to_string());
            let second_entry = self.second_order.entry(key).or_default();
            *second_entry.entry(to_file.to_string()).or_insert(0) += 1;
        }

        let co_entry = self.co_occurrence.entry(from_file.to_string()).or_default();
        *co_entry.entry(to_file.to_string()).or_insert(0) += 1;

        self.history.push(from_file.to_string());
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }

    pub fn record_git_commit_cluster(&mut self, files: &[String]) {
        for i in 0..files.len() {
            for j in 0..files.len() {
                if i != j {
                    let entry = self.co_occurrence.entry(files[i].clone()).or_default();
                    *entry.entry(files[j].clone()).or_insert(0) += 2;
                }
            }
        }
    }

    pub fn predict_next_targets(&self, current_file: &str, top_k: usize) -> Vec<(String, f64)> {
        let mut score_map: HashMap<String, f64> = HashMap::new();

        if let Some(prev) = self.history.last() {
            let key = (prev.clone(), current_file.to_string());
            if let Some(second_counts) = self.second_order.get(&key) {
                let total: usize = second_counts.values().sum();
                if total > 0 {
                    for (file, &cnt) in second_counts {
                        let p = (cnt as f64) / (total as f64);
                        *score_map.entry(file.clone()).or_default() += p * 0.5;
                    }
                }
            }
        }

        if let Some(first_counts) = self.first_order.get(current_file) {
            let total: usize = first_counts.values().sum();
            if total > 0 {
                for (file, &cnt) in first_counts {
                    let p = (cnt as f64) / (total as f64);
                    *score_map.entry(file.clone()).or_default() += p * 0.3;
                }
            }
        }

        if let Some(co_counts) = self.co_occurrence.get(current_file) {
            let total: usize = co_counts.values().sum();
            if total > 0 {
                for (file, &cnt) in co_counts {
                    let p = (cnt as f64) / (total as f64);
                    *score_map.entry(file.clone()).or_default() += p * 0.2;
                }
            }
        }

        let mut sorted: Vec<(String, f64)> = score_map.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(top_k);
        sorted
    }
}

pub struct SpeculativePlanner;

impl SpeculativePlanner {
    pub fn plan_idle_compilation(
        model: &MarkovTransitionModel,
        modified_file: &str,
    ) -> Vec<String> {
        let predictions = model.predict_next_targets(modified_file, 5);
        predictions
            .into_iter()
            .filter(|(_, prob)| *prob >= 0.15)
            .map(|(target, _)| target)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markov_speculative_predictions() {
        let mut model = MarkovTransitionModel::new();
        model.record_transition("src/auth.rs", "src/user.rs");
        model.record_transition("src/auth.rs", "src/user.rs");
        model.record_transition("src/auth.rs", "src/token.rs");

        let predictions = model.predict_next_targets("src/auth.rs", 2);
        assert!(!predictions.is_empty());
        assert_eq!(predictions[0].0, "src/user.rs");

        let planned = SpeculativePlanner::plan_idle_compilation(&model, "src/auth.rs");
        assert!(planned.contains(&"src/user.rs".to_string()));
    }

    #[test]
    fn test_git_commit_cluster_boost() {
        let mut model = MarkovTransitionModel::new();
        model.record_git_commit_cluster(&[
            "crates/core/src/lib.rs".to_string(),
            "crates/cli/src/main.rs".to_string(),
            "crates/cas/src/lib.rs".to_string(),
        ]);

        let targets = model.predict_next_targets("crates/core/src/lib.rs", 3);
        assert_eq!(targets.len(), 2);
        let names: Vec<_> = targets.into_iter().map(|(n, _)| n).collect();
        assert!(names.contains(&"crates/cli/src/main.rs".to_string()));
        assert!(names.contains(&"crates/cas/src/lib.rs".to_string()));
    }
}
