#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

use fish_core::{BuildBackend, FingerprintUtils, TaskDagBuilder};
use fish_executor::{CacheEntry, Task};
use fish_graph::BuildGraph;

pub mod config;
pub mod fingerprint;
pub mod toolchain;

pub use config::{PackagingType, PexConfig, PyProjectConfig, PyTaskSpec, PythonRunner};
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
    Graph(#[from] fish_graph::GraphError),
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

    pub fn build_task_graph(
        &self,
        config: &PyProjectConfig,
        project_dir: &Path,
    ) -> Result<BuildGraph<Task>, PyBackendError> {
        let mut graph = BuildGraph::new();
        let namespace = FingerprintUtils::compute_namespace(project_dir);
        let base_fp = fingerprint::compute_py_fingerprint(project_dir, &config.source_dirs)?;

        let order = TaskDagBuilder::resolve_dag_order(
            &config.tasks,
            |task| &task.name,
            |task| &task.depends_on,
        )
        .map_err(PyBackendError::Config)?;

        let mut node_map: HashMap<String, fish_graph::NodeId> = HashMap::new();
        let mut dep_fps: HashMap<String, String> = HashMap::new();

        for idx in order {
            let task_spec = &config.tasks[idx];
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

            let task_fp = FingerprintUtils::combine_fingerprints(
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
                key: FingerprintUtils::format_cache_key("py", &namespace, "task", &task_spec.name),
                fingerprint: task_fp.clone(),
            });

            let node_id = graph.add_node(task);
            node_map.insert(task_spec.name.clone(), node_id);
            dep_fps.insert(task_spec.name.clone(), task_fp);
        }

        for task in &config.tasks {
            let node_id = node_map[&task.name];
            for dep in &task.depends_on {
                graph.add_dependency(node_map[dep], node_id)?;
            }
        }

        if let Some(PackagingType::Pex) = config.packaging {
            let pex_cfg = config.pex.clone().unwrap_or_default();
            let default_pex_name = format!("{}.pex", config.name);
            let out_file = project_dir
                .join("dist")
                .join(pex_cfg.output_pex.as_deref().unwrap_or(&default_pex_name));
            let pex_spec = self
                .toolchain
                .build_pex_command(&pex_cfg, project_dir, &out_file);
            let pex_fp = FingerprintUtils::combine_fingerprints(
                &format!("py:pex:{base_fp}"),
                &dep_fps.values().cloned().collect::<Vec<_>>(),
            );
            let pex_task = Task::new(
                format!("{}:pex", config.name),
                pex_spec.command_line(),
                pex_spec,
            )
            .with_cache(CacheEntry {
                key: FingerprintUtils::format_cache_key("py", &namespace, "pex", &config.name),
                fingerprint: pex_fp,
            });
            let pex_node = graph.add_node(pex_task);
            if let Some(last_node) = node_map.get("build").or_else(|| node_map.get("test")) {
                graph.add_dependency(*last_node, pex_node)?;
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

    #[test]
    fn test_py_backend_pex_packaging() {
        let dir = tempdir().unwrap();
        let mut config =
            PyProjectConfig::discover_or_default(dir.path()).unwrap_or_else(|_| PyProjectConfig {
                name: "analytics-worker".to_string(),
                runner: Some(PythonRunner::Uv),
                packaging: Some(PackagingType::Pex),
                pex: Some(PexConfig {
                    entry_point: Some("analytics.main:app".to_string()),
                    output_pex: Some("analytics.pex".to_string()),
                    interpreter_constraint: Some(">=3.11".to_string()),
                    platforms: vec!["manylinux2014_x86_64".to_string()],
                    inherit_path: Some("prefer".to_string()),
                    include_tools: true,
                }),
                tasks: vec![PyTaskSpec {
                    name: "build".to_string(),
                    command: Some("uv".to_string()),
                    args: vec!["build".to_string()],
                    depends_on: vec![],
                }],
                source_dirs: vec![],
            });
        config.packaging = Some(PackagingType::Pex);
        let backend = PyBackend::new();
        let graph = backend.build_task_graph(&config, dir.path()).unwrap();
        assert!(graph.len() >= 2);
    }
}
