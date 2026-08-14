#![forbid(unsafe_code)]

use std::path::Path;
use thiserror::Error;

use forge_core::BuildBackend;
use forge_executor::{CacheEntry, CommandSpec, Task};
use forge_graph::BuildGraph;

pub mod config;
pub mod fingerprint;
pub mod toolchain;

pub use config::GoProjectConfig;
pub use toolchain::GoToolchain;

#[derive(Debug, Error)]
pub enum GoBackendError {
    #[error("toolchain error: {0}")]
    Toolchain(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("graph error: {0}")]
    Graph(#[from] forge_graph::GraphError),
}

#[derive(Debug, Clone)]
pub struct GoBackend {
    pub toolchain: GoToolchain,
}

impl BuildBackend for GoBackend {
    fn name(&self) -> &'static str {
        "go"
    }
}

impl GoBackend {
    pub fn new() -> Result<Self, GoBackendError> {
        let toolchain = GoToolchain::detect().map_err(GoBackendError::Toolchain)?;
        Ok(Self { toolchain })
    }

    pub fn with_toolchain(toolchain: GoToolchain) -> Self {
        Self { toolchain }
    }

    pub fn create_tasks_from_config(
        &self,
        config: &GoProjectConfig,
        project_dir: &Path,
        output_dir: &Path,
    ) -> Result<BuildGraph<Task>, GoBackendError> {
        let mut graph = BuildGraph::new();
        std::fs::create_dir_all(output_dir)?;

        let mut hasher = blake3::Hasher::new();
        hasher.update(project_dir.to_string_lossy().as_bytes());
        let namespace = hasher.finalize().to_hex().to_string()[..12].to_string();

        let fp =
            fingerprint::compute_go_fingerprint(project_dir, &self.toolchain.version, &config.tags)
                .unwrap_or_else(|_| "no_fp".to_string());

        let vet_args = self.toolchain.vet_args(&config.package_path);
        let vet_spec = CommandSpec::new(&self.toolchain.executable)
            .args(vet_args)
            .cwd(project_dir);
        let vet_task = Task::new(
            format!("go vet {}", config.name),
            vet_spec.command_line(),
            vet_spec,
        );
        let vet_node_id = graph.add_node(vet_task);

        let binary_ext = if cfg!(windows) { ".exe" } else { "" };
        let default_out = output_dir
            .join(format!("{}{}", config.name, binary_ext))
            .to_string_lossy()
            .to_string();

        let out_path = config
            .output_binary
            .as_deref()
            .unwrap_or(default_out.as_str());

        let build_args = self.toolchain.build_args(
            &config.package_path,
            Some(out_path),
            &config.tags,
            config.ldflags.as_deref(),
            config.gcflags.as_deref(),
        );

        let build_spec = CommandSpec::new(&self.toolchain.executable)
            .args(build_args)
            .cwd(project_dir);

        let build_cache = CacheEntry {
            key: format!("go/build/{}/{}", namespace, config.name),
            fingerprint: fp.clone(),
        };

        let build_task = Task::new(
            format!("go build {}", config.name),
            build_spec.command_line(),
            build_spec,
        )
        .with_cache(build_cache);

        let build_node_id = graph.add_node(build_task);
        graph.add_dependency(vet_node_id, build_node_id)?;

        if config.run_tests {
            let test_args = self.toolchain.test_args(&config.package_path, &config.tags);
            let test_spec = CommandSpec::new(&self.toolchain.executable)
                .args(test_args)
                .cwd(project_dir);

            let test_cache = CacheEntry {
                key: format!("go/test/{}/{}", namespace, config.name),
                fingerprint: fp,
            };

            let test_task = Task::new(
                format!("go test {}", config.name),
                test_spec.command_line(),
                test_spec,
            )
            .with_cache(test_cache);

            let test_node_id = graph.add_node(test_task);
            graph.add_dependency(build_node_id, test_node_id)?;
        }

        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_go_backend_task_graph_construction() {
        let dummy_toolchain = GoToolchain {
            executable: "go".to_string(),
            version: "go version go1.22.0 windows/amd64".to_string(),
        };

        let backend = GoBackend::with_toolchain(dummy_toolchain);

        let config = GoProjectConfig {
            name: "my_go_app".to_string(),
            package_path: "./...".to_string(),
            tags: vec!["netgo".to_string()],
            ldflags: None,
            gcflags: None,
            run_tests: true,
            output_binary: None,
        };

        let temp = tempdir().unwrap();
        let graph = backend
            .create_tasks_from_config(&config, temp.path(), &temp.path().join("build"))
            .unwrap();

        assert_eq!(graph.len(), 3);
        assert_eq!(backend.name(), "go");

        let topo = graph.topological_order();
        assert_eq!(topo.len(), 3);

        let first = graph.node(topo[0]).unwrap();
        let last = graph.node(topo[2]).unwrap();
        assert!(first.payload.label.starts_with("go vet"));
        assert!(last.payload.label.starts_with("go test"));
    }
}
