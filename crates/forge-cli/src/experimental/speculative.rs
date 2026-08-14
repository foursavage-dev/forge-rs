#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct MarkovTransitionModel {
    transitions: HashMap<String, HashMap<String, usize>>,
}

impl MarkovTransitionModel {
    pub fn new() -> Self {
        Self {
            transitions: HashMap::new(),
        }
    }

    pub fn record_transition(&mut self, from_file: &str, to_file: &str) {
        let entry = self.transitions.entry(from_file.to_string()).or_default();
        *entry.entry(to_file.to_string()).or_insert(0) += 1;
    }

    pub fn predict_next_targets(&self, current_file: &str, top_k: usize) -> Vec<(String, f64)> {
        if let Some(next_counts) = self.transitions.get(current_file) {
            let total: usize = next_counts.values().sum();
            if total == 0 {
                return Vec::new();
            }

            let mut probabilities: Vec<(String, f64)> = next_counts
                .iter()
                .map(|(file, &count)| (file.clone(), (count as f64) / (total as f64)))
                .collect();

            probabilities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            probabilities.truncate(top_k);
            probabilities
        } else {
            Vec::new()
        }
    }
}

pub struct SpeculativePlanner;

impl SpeculativePlanner {
    pub fn plan_idle_compilation(
        model: &MarkovTransitionModel,
        modified_file: &str,
    ) -> Vec<String> {
        let predictions = model.predict_next_targets(modified_file, 3);
        predictions
            .into_iter()
            .filter(|(_, prob)| *prob >= 0.3)
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
        assert_eq!(predictions.len(), 2);
        assert_eq!(predictions[0].0, "src/user.rs");
        assert!(predictions[0].1 > 0.6);

        let planned = SpeculativePlanner::plan_idle_compilation(&model, "src/auth.rs");
        assert_eq!(planned, vec!["src/user.rs", "src/token.rs"]);
    }
}
