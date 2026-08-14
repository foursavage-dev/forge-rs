#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct PredictiveStats {
    pub touch_counts: HashMap<PathBuf, usize>,
    pub last_touch: HashMap<PathBuf, Instant>,
    pub speculative_hits: usize,
}

#[derive(Debug, Clone)]
pub struct PredictiveEngine {
    stats: Arc<Mutex<PredictiveStats>>,
    enabled: bool,
}

impl PredictiveEngine {
    pub fn new(enabled: bool) -> Self {
        Self {
            stats: Arc::new(Mutex::new(PredictiveStats::default())),
            enabled,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn record_touch(&self, path: &Path) {
        if !self.enabled {
            return;
        }
        if let Ok(mut stats) = self.stats.lock() {
            let path_buf = path.to_path_buf();
            *stats.touch_counts.entry(path_buf.clone()).or_insert(0) += 1;
            stats.last_touch.insert(path_buf, Instant::now());
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
                }
            }
        }
        affected
    }

    pub fn speculative_warmup_candidates(
        &self,
        all_packages: &[String],
        affected: &HashSet<String>,
    ) -> Vec<String> {
        all_packages
            .iter()
            .filter(|pkg| !affected.contains(*pkg))
            .cloned()
            .collect()
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
        engine.record_touch(Path::new("src/main.rs"));
        assert_eq!(engine.hits(), 0);
    }

    #[test]
    fn test_predictive_engine_enabled_and_candidates() {
        let engine = PredictiveEngine::new(true);
        assert!(engine.is_enabled());

        let path = PathBuf::from("crates/core/src/lib.rs");
        engine.record_touch(&path);

        let mut changed = HashSet::new();
        changed.insert(path);

        let roots = vec![
            ("core".to_string(), PathBuf::from("crates/core")),
            ("cli".to_string(), PathBuf::from("crates/cli")),
        ];

        let affected = engine.predict_affected_packages(&changed, &roots);
        assert!(affected.contains("core"));
        assert!(!affected.contains("cli"));

        let all_pkgs = vec!["core".to_string(), "cli".to_string()];
        let warmup = engine.speculative_warmup_candidates(&all_pkgs, &affected);
        assert_eq!(warmup, vec!["cli".to_string()]);

        engine.record_speculative_hit();
        assert_eq!(engine.hits(), 1);
    }
}
