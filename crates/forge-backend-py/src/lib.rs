#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

use forge_core::BuildBackend;
use forge_executor::{CacheEntry, Task};
use forge_graph::BuildGraph;

pub mod config;
pub mod fingerprint;
pub mod toolchain;

pub use config::{PyProjectConfig, PyTaskSpec, PythonRunner};
pub use toolchain::PyToolchain;

#[derive(Debug, Error)]
pub enum PyBackendError {
    #[error("toolchain error: {0}")]
    Toolchain(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("graph error: {0}")]
    Graph(#[from] forge_graph::GraphError),
}

#[derive(Debug, Clone, Default)]
pub struct PyBackend {
    pub toolchain: PyToolchain,
}

impl BuildBackend for PyBackend {
    fn name(&self) -> &'static str {
        "python"
    }
}

impl PyBackend {
    pub fn new() -> Self {
        Self {
            toolchain: PyToolchain::new(),
        }
    }

    pub fn with_toolchain(toolchain: PyToolchain) -> Self {
        Self { toolchain }
    }

    fn combined_fingerprint(prefix: &str, member_fps: &[String]) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(prefix.as_bytes());
        let mut sorted = member_fps.to_vec();
        sorted.sort();
        for fp in &sorted {
            hasher.update(b":");
            hasher.update(fp.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }

    pub fn build_task_graph(
        &self,
        config: &PyProjectConfig,
        project_dir: &Path,
    ) -> Result<BuildGraph<Task>, PyBackendError> {
        let mut graph = BuildGraph::new();

        let mut hasher = blake3::Hasher::new();
        hasher.update(project_dir.to_string_lossy().as_bytes());
        let namespace = hasher.finalize().to_hex().to_string()[..12].to_string();

        let base_fp = fingerprint::compute_py_fingerprint(project_dir, &config.source_dirs)?;

        let mut node_map: HashMap<String, forge_graph::NodeId> = HashMap::new();
        let mut dep_fps: HashMap<String, String> = HashMap::new();

        while node_map.len() < config.tasks.len() {
            let ready: Vec<&PyTaskSpec> = config
                .tasks
                .iter()
                .filter(|task| !node_map.contains_key(&task.name))
                .filter(|task| task.depends_on.iter().all(|dep| node_map.contains_key(dep)))
                .collect();

            if ready.is_empty() {
                let unresolved: Vec<&str> = config
                    .tasks
                    .iter()
                    .filter(|task| !node_map.contains_key(&task.name))
                    .map(|task| task.name.as_str())
                    .collect();
                return Err(PyBackendError::Config(format!(
                    "dependency cycle or unknown dependency among tasks: {}",
                    unresolved.join(", ")
                )));
            }

            for task_spec in ready {
                let mut member_fps = Vec::new();
                for dep in &task_spec.depends_on {
                    member_fps.push(
                        dep_fps
                            .get(dep)
                            .ok_or_else(|| {
                                PyBackendError::Config(format!(
                                    "task `{}` depends on unknown task `{}`",
                                    task_spec.name, dep
                                ))
                            })?
                            .clone(),
                    );
                }

                let task_fp = Self::combined_fingerprint(
                    &format!("py:{base_fp}:{}", task_spec.name),
                    &member_fps,
                );

                let spec = self.toolchain.build_command(
                    task_spec,
                    config.runner.unwrap_or_default(),
                    project_dir,
                );

                let task = Task::new(
                    format!("{}:{}", config.name, task_spec.name),
                    spec.command_line(),
                    spec,
                )
                .with_cache(CacheEntry {
                    key: format!("py/{}/{}", namespace, task_spec.name),
                    fingerprint: task_fp.clone(),
                });

                let node_id = graph.add_node(task);
                node_map.insert(task_spec.name.clone(), node_id);
                dep_fps.insert(task_spec.name.clone(), task_fp);
            }
        }

        for task in &config.tasks {
            let node_id = node_map[&task.name];
            for dep in &task.depends_on {
                graph.add_dependency(node_map[dep], node_id)?;
            }
        }

        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_py_backend_task_graph_construction() {
        let dir = tempdir().unwrap();
        let pyproject = r#"[project]
name = "my-service"
version = "0.1.0"
"#;
        fs::write(dir.path().join("pyproject.toml"), pyproject).unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src").join("main.py"), "print('hello')").unwrap();

        let config = PyProjectConfig::discover_or_default(dir.path()).unwrap();
        let backend = PyBackend::new();
        let graph = backend.build_task_graph(&config, dir.path()).unwrap();

        assert_eq!(graph.len(), 4);
        let topo = graph.topological_order();
        assert_eq!(topo.len(), 4);
    }
}
