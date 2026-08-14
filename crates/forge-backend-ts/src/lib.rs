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

pub use config::{PackageManager, TsProjectConfig, TsTaskSpec};
pub use toolchain::TsToolchain;

#[derive(Debug, Error)]
pub enum TsBackendError {
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
pub struct TsBackend {
    pub toolchain: TsToolchain,
}

impl BuildBackend for TsBackend {
    fn name(&self) -> &'static str {
        "typescript"
    }
}

impl TsBackend {
    pub fn new() -> Self {
        Self {
            toolchain: TsToolchain::new(),
        }
    }

    pub fn with_toolchain(toolchain: TsToolchain) -> Self {
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
        config: &TsProjectConfig,
        project_dir: &Path,
    ) -> Result<BuildGraph<Task>, TsBackendError> {
        let mut graph = BuildGraph::new();

        let mut hasher = blake3::Hasher::new();
        hasher.update(project_dir.to_string_lossy().as_bytes());
        let namespace = hasher.finalize().to_hex().to_string()[..12].to_string();

        let base_fp = fingerprint::compute_ts_fingerprint(project_dir, &config.source_dirs)?;

        let mut node_map: HashMap<String, forge_graph::NodeId> = HashMap::new();
        let mut dep_fps: HashMap<String, String> = HashMap::new();

        while node_map.len() < config.tasks.len() {
            let ready: Vec<&TsTaskSpec> = config
                .tasks
                .iter()
                .filter(|task| !node_map.contains_key(&task.name))
                .filter(|task| task.depends_on.iter().all(|dep| node_map.contains_key(dep)))
                .collect();

            if ready.is_empty() {
                let known: std::collections::HashSet<&str> =
                    config.tasks.iter().map(|t| t.name.as_str()).collect();
                for task in config
                    .tasks
                    .iter()
                    .filter(|t| !node_map.contains_key(&t.name))
                {
                    for dep in &task.depends_on {
                        if !known.contains(dep.as_str()) {
                            return Err(TsBackendError::Config(format!(
                                "task `{}` depends on unknown task `{}`",
                                task.name, dep
                            )));
                        }
                    }
                }
                let cycle: Vec<&str> = config
                    .tasks
                    .iter()
                    .filter(|t| !node_map.contains_key(&t.name))
                    .map(|t| t.name.as_str())
                    .collect();
                return Err(TsBackendError::Config(format!(
                    "dependency cycle among tasks: {}",
                    cycle.join(" -> ")
                )));
            }

            for task_spec in ready {
                let mut member_fps = Vec::new();
                for dep in &task_spec.depends_on {
                    member_fps.push(
                        dep_fps
                            .get(dep)
                            .ok_or_else(|| {
                                TsBackendError::Config(format!(
                                    "task `{}` depends on unknown task `{}`",
                                    task_spec.name, dep
                                ))
                            })?
                            .clone(),
                    );
                }

                let task_fp = Self::combined_fingerprint(
                    &format!("ts:{base_fp}:{}", task_spec.name),
                    &member_fps,
                );

                let spec = self.toolchain.build_command(
                    task_spec,
                    config.package_manager.unwrap_or_default(),
                    project_dir,
                );

                let task = Task::new(
                    format!("{}:{}", config.name, task_spec.name),
                    spec.command_line(),
                    spec,
                )
                .with_cache(CacheEntry {
                    key: format!("ts/{}/{}", namespace, task_spec.name),
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
    fn test_ts_backend_task_graph_construction() {
        let dir = tempdir().unwrap();
        let pkg_json = r#"{
            "name": "web-app",
            "scripts": {
                "typecheck": "tsc --noEmit",
                "build": "vite build",
                "test": "vitest run"
            }
        }"#;
        fs::write(dir.path().join("package.json"), pkg_json).unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        fs::write(
            dir.path().join("src").join("index.ts"),
            "console.log('hello');",
        )
        .unwrap();

        let config = TsProjectConfig::discover_or_default(dir.path()).unwrap();
        let backend = TsBackend::new();
        let graph = backend.build_task_graph(&config, dir.path()).unwrap();

        assert_eq!(graph.len(), 3);
        let topo = graph.topological_order();
        assert_eq!(topo.len(), 3);

        let first = graph.node(topo[0]).unwrap();
        let last = graph.node(topo[2]).unwrap();
        assert!(first.payload.label.ends_with(":typecheck"));
        assert!(last.payload.label.ends_with(":test"));
        assert_eq!(backend.name(), "typescript");

        let cache = last.payload.cache.as_ref().unwrap();
        assert!(cache.key.starts_with("ts/"), "got: {}", cache.key);
        assert_eq!(cache.fingerprint.len(), 64);
    }

    #[test]
    fn unknown_dependency_is_an_error() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("forge.ts.json"),
            r#"{
                "name": "broken",
                "tasks": [
                    { "name": "build", "args": ["run", "build"], "depends_on": ["missing"] }
                ]
            }"#,
        )
        .unwrap();

        let config = TsProjectConfig::discover_or_default(dir.path()).unwrap();
        let backend = TsBackend::new();
        let error = backend.build_task_graph(&config, dir.path()).unwrap_err();
        assert!(error.to_string().contains("unknown task"), "got: {error}");
    }
}
