#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIMatrix {
    pub jobs: Vec<CIJob>,
    pub dependencies: HashMap<String, Vec<String>>,
    pub cache_config: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIJob {
    pub id: String,
    pub name: String,
    pub backend: String,
    pub commands: Vec<String>,
    pub artifacts: Vec<String>,
    pub dependencies: Vec<String>,
    pub cache_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub remote_url: Option<String>,
    pub key_prefix: String,
}

impl Default for CIMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl CIMatrix {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            dependencies: HashMap::new(),
            cache_config: CacheConfig {
                enabled: true,
                remote_url: None,
                key_prefix: "fish".to_string(),
            },
        }
    }

    pub fn add_job(&mut self, job: CIJob) {
        self.dependencies
            .insert(job.id.clone(), job.dependencies.clone());
        self.jobs.push(job);
    }

    pub fn topological_sort(&self) -> Vec<String> {
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for job in &self.jobs {
            self.visit(&job.id, &mut sorted, &mut visited);
        }

        sorted
    }

    fn visit(
        &self,
        job_id: &str,
        sorted: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if visited.contains(job_id) {
            return;
        }

        visited.insert(job_id.to_string());

        if let Some(deps) = self.dependencies.get(job_id) {
            for dep in deps {
                self.visit(dep, sorted, visited);
            }
        }

        sorted.push(job_id.to_string());
    }

    pub fn get_parallel_levels(&self) -> Vec<Vec<String>> {
        let sorted = self.topological_sort();
        let mut levels = Vec::new();
        let mut current_level = Vec::new();
        let mut completed = std::collections::HashSet::new();

        for job_id in &sorted {
            let deps = self.dependencies.get(job_id).cloned().unwrap_or_default();

            // Check if all dependencies are completed
            let can_run = deps.iter().all(|dep| completed.contains(dep));

            if can_run {
                current_level.push(job_id.clone());
                completed.insert(job_id.clone());
            } else {
                if !current_level.is_empty() {
                    levels.push(current_level);
                    current_level = Vec::new();
                }
                current_level.push(job_id.clone());
                completed.insert(job_id.clone());
            }
        }

        if !current_level.is_empty() {
            levels.push(current_level);
        }

        levels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CIJob;

    #[test]
    fn test_matrix_creation() {
        let matrix = CIMatrix::new();
        assert_eq!(matrix.jobs.len(), 0);
    }

    #[test]
    fn test_job_addition() {
        let mut matrix = CIMatrix::new();
        let job = CIJob {
            id: "test-job".to_string(),
            name: "Test Job".to_string(),
            backend: "rust".to_string(),
            commands: vec!["cargo build".to_string()],
            artifacts: vec![],
            dependencies: vec![],
            cache_key: "test-key".to_string(),
        };

        matrix.add_job(job);
        assert_eq!(matrix.jobs.len(), 1);
    }

    #[test]
    fn test_topological_sort() {
        let mut matrix = CIMatrix::new();

        matrix.add_job(CIJob {
            id: "job1".to_string(),
            name: "Job 1".to_string(),
            backend: "rust".to_string(),
            commands: vec![],
            artifacts: vec![],
            dependencies: vec![],
            cache_key: "key1".to_string(),
        });

        matrix.add_job(CIJob {
            id: "job2".to_string(),
            name: "Job 2".to_string(),
            backend: "rust".to_string(),
            commands: vec![],
            artifacts: vec![],
            dependencies: vec!["job1".to_string()],
            cache_key: "key2".to_string(),
        });

        let sorted = matrix.topological_sort();
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0], "job1");
        assert_eq!(sorted[1], "job2");
    }
}
