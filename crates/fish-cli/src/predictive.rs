#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Markov-chain based Speculative Pre-Compilation Engine
/// Tracks file modifications and builds a transition matrix to predict
/// which package a developer is likely to modify next.
#[derive(Debug, Clone, Default)]
pub struct PredictiveStats {
    pub touch_counts: HashMap<PathBuf, usize>,
    pub last_touch: HashMap<PathBuf, Instant>,
    // Transition matrix mapping: Package A -> (Package B -> Frequency)
    pub markov_transitions: HashMap<String, HashMap<String, usize>>,
    pub last_modified_pkg: Option<String>,
    pub speculative_hits: usize,
}

#[derive(Debug, Clone)]
pub struct PredictiveEngine {
    stats: Arc<Mutex<PredictiveStats>>,
    enabled: bool,
    confidence_threshold: f64,
}

impl PredictiveEngine {
    pub fn new(enabled: bool) -> Self {
        Self {
            stats: Arc::new(Mutex::new(PredictiveStats::default())),
            enabled,
            confidence_threshold: 0.35, // 35% probability threshold to trigger speculative warmup
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn record_touch(&self, path: &Path, package_name: Option<String>) {
        if !self.enabled {
            return;
        }
        if let Ok(mut stats) = self.stats.lock() {
            let path_buf = path.to_path_buf();
            *stats.touch_counts.entry(path_buf.clone()).or_insert(0) += 1;
            stats.last_touch.insert(path_buf, Instant::now());

            if let Some(current_pkg) = package_name {
                if let Some(prev_pkg) = stats.last_modified_pkg.clone()
                    && prev_pkg != current_pkg
                {
                    let transitions = stats
                        .markov_transitions
                        .entry(prev_pkg)
                        .or_insert_with(HashMap::new);
                    *transitions.entry(current_pkg.clone()).or_insert(0) += 1;
                }
                stats.last_modified_pkg = Some(current_pkg);
            }
        }
    }

    pub fn predict_affected_packages(
        &self,
        changed_paths: &HashSet<PathBuf>,
        package_roots: &[(String, PathBuf)],
    ) -> HashSet<String> {
        let mut affected = HashSet::new();
        for path in changed_paths {
            for (pkg_name, root) in package_roots {
                if path.starts_with(root) {
                    affected.insert(pkg_name.clone());
                    // Also record to feed the Markov Chain during normal execution
                    self.record_touch(path, Some(pkg_name.clone()));
                }
            }
        }
        affected
    }

    /// Uses the Markov Chain transition matrix to predict the most likely NEXT packages
    pub fn speculative_warmup_candidates(
        &self,
        all_packages: &[String],
        affected: &HashSet<String>,
    ) -> Vec<String> {
        if !self.enabled || affected.is_empty() {
            return vec![];
        }

        let mut candidates = Vec::new();
        if let Ok(stats) = self.stats.lock() {
            for pkg in affected {
                if let Some(transitions) = stats.markov_transitions.get(pkg) {
                    let total_transitions: usize = transitions.values().sum();
                    if total_transitions == 0 {
                        continue;
                    }

                    for (next_pkg, &count) in transitions {
                        let probability = count as f64 / total_transitions as f64;
                        if probability >= self.confidence_threshold && !affected.contains(next_pkg)
                        {
                            candidates.push(next_pkg.clone());
                        }
                    }
                }
            }
        }

        // Deduplicate and filter candidates that actually exist in the workspace
        let mut unique_candidates: Vec<String> = candidates
            .into_iter()
            .filter(|p| all_packages.contains(p))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        unique_candidates.sort(); // Stable output
        unique_candidates
    }

    pub fn record_speculative_hit(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.speculative_hits += 1;
        }
    }

    pub fn hits(&self) -> usize {
        self.stats.lock().map(|s| s.speculative_hits).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predictive_engine_disabled() {
        let engine = PredictiveEngine::new(false);
        assert!(!engine.is_enabled());
        engine.record_touch(Path::new("src/main.rs"), None);
        assert_eq!(engine.hits(), 0);
    }

    #[test]
    fn test_markov_chain_speculation() {
        let engine = PredictiveEngine::new(true);
        assert!(engine.is_enabled());

        // Simulate developer workflow: core -> utils -> cli
        engine.record_touch(
            &PathBuf::from("crates/core/src/lib.rs"),
            Some("core".to_string()),
        );
        engine.record_touch(
            &PathBuf::from("crates/utils/src/lib.rs"),
            Some("utils".to_string()),
        );
        engine.record_touch(
            &PathBuf::from("crates/cli/src/main.rs"),
            Some("cli".to_string()),
        );

        // Simulate a second time to build probability weight
        engine.record_touch(
            &PathBuf::from("crates/core/src/lib.rs"),
            Some("core".to_string()),
        );
        engine.record_touch(
            &PathBuf::from("crates/utils/src/lib.rs"),
            Some("utils".to_string()),
        );

        let mut changed = HashSet::new();
        changed.insert("core".to_string());

        let all_pkgs = vec![
            "core".to_string(),
            "utils".to_string(),
            "cli".to_string(),
            "backend".to_string(),
        ];

        // Because core -> utils happened twice, probability is 1.0 > threshold (0.35)
        let candidates = engine.speculative_warmup_candidates(&all_pkgs, &changed);
        assert_eq!(candidates, vec!["utils".to_string()]);
    }
}
