#![forbid(unsafe_code)]

use std::path::Path;
use thiserror::Error;

use fish_core::{BuildBackend, FingerprintUtils};
use fish_executor::{CacheEntry, CommandSpec, Task};
use fish_graph::BuildGraph;

pub mod config;
pub mod fingerprint;
pub mod toolchain;

pub use config::{DotnetProjectConfig, DotnetTargetFramework};
pub use toolchain::{DotnetCompiler, DotnetToolchain};

#[derive(Debug, Error)]
pub enum DotnetBackendError {
    #[error("toolchain error: {0}")]
    Toolchain(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("graph error: {0}")]
    Graph(#[from] fish_graph::GraphError),
    #[error("parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone)]
pub struct DotnetBackend {
    pub toolchain: DotnetToolchain,
}

impl BuildBackend for DotnetBackend {
    fn name(&self) -> &'static str {
        "dotnet"
    }
}

impl DotnetBackend {
    pub fn new() -> Result<Self, DotnetBackendError> {
        let toolchain =
            DotnetToolchain::detect().map_err(|e| DotnetBackendError::Toolchain(e.to_string()))?;
        Ok(Self { toolchain })
    }

    pub fn with_toolchain(toolchain: DotnetToolchain) -> Self {
        Self { toolchain }
    }

    pub fn create_tasks_from_config(
        &self,
        config: &DotnetProjectConfig,
        project_dir: &Path,
        output_dir: &Path,
    ) -> Result<BuildGraph<Task>, DotnetBackendError> {
        let mut graph = BuildGraph::new();
        std::fs::create_dir_all(output_dir)?;
        let namespace = FingerprintUtils::compute_namespace(project_dir);

        let fp = fingerprint::compute_dotnet_fingerprint(
            project_dir,
            &self.toolchain.dotnet_version,
            &config.target_framework,
        )
        .unwrap_or_else(|_| "no_fp".to_string());

        let restore_args = vec!["restore".to_string()];
        let restore_spec = CommandSpec::new(&self.toolchain.executable)
            .args(restore_args)
            .cwd(project_dir);
        let restore_cache = CacheEntry {
            key: FingerprintUtils::format_cache_key(
                "dotnet",
                &namespace,
                "restore",
                &config.project_name,
            ),
            fingerprint: fp.clone(),
        };
        let restore_task = Task::new(
            format!("dotnet restore {}", config.project_name),
            restore_spec.command_line(),
            restore_spec,
        )
        .with_cache(restore_cache);
        let restore_node_id = graph.add_node(restore_task);

        let mut build_args = vec!["build".to_string()];
        if config.release {
            build_args.push("--configuration".to_string());
            build_args.push("Release".to_string());
        } else {
            build_args.push("--configuration".to_string());
            build_args.push("Debug".to_string());
        }

        if let Some(output) = &config.output_path {
            build_args.push("--output".to_string());
            build_args.push(output.clone());
        }

        let build_spec = CommandSpec::new(&self.toolchain.executable)
            .args(build_args)
            .cwd(project_dir);
        let build_cache = CacheEntry {
            key: FingerprintUtils::format_cache_key(
                "dotnet",
                &namespace,
                "build",
                &config.project_name,
            ),
            fingerprint: fp.clone(),
        };
        let build_task = Task::new(
            format!("dotnet build {}", config.project_name),
            build_spec.command_line(),
            build_spec,
        )
        .with_cache(build_cache);
        let build_node_id = graph.add_node(build_task);
        graph.add_dependency(restore_node_id, build_node_id)?;

        if config.run_tests {
            let test_args = vec!["test".to_string()];
            let test_spec = CommandSpec::new(&self.toolchain.executable)
                .args(test_args)
                .cwd(project_dir);
            let test_cache = CacheEntry {
                key: FingerprintUtils::format_cache_key(
                    "dotnet",
                    &namespace,
                    "test",
                    &config.project_name,
                ),
                fingerprint: fp.clone(),
            };
            let test_task = Task::new(
                format!("dotnet test {}", config.project_name),
                test_spec.command_line(),
                test_spec,
            )
            .with_cache(test_cache);
            let test_node_id = graph.add_node(test_task);
            graph.add_dependency(build_node_id, test_node_id)?;
        }

        if config.publish {
            let mut publish_args = vec!["publish".to_string()];
            if config.release {
                publish_args.push("--configuration".to_string());
                publish_args.push("Release".to_string());
            }

            if let Some(runtime) = &config.runtime {
                publish_args.push("--runtime".to_string());
                publish_args.push(runtime.clone());
            }

            if let Some(output) = &config.output_path {
                publish_args.push("--output".to_string());
                publish_args.push(output.clone());
            }

            let publish_spec = CommandSpec::new(&self.toolchain.executable)
                .args(publish_args)
                .cwd(project_dir);
            let publish_cache = CacheEntry {
                key: FingerprintUtils::format_cache_key(
                    "dotnet",
                    &namespace,
                    "publish",
                    &config.project_name,
                ),
                fingerprint: fp,
            };
            let publish_task = Task::new(
                format!("dotnet publish {}", config.project_name),
                publish_spec.command_line(),
                publish_spec,
            )
            .with_cache(publish_cache);
            let publish_node_id = graph.add_node(publish_task);
            graph.add_dependency(build_node_id, publish_node_id)?;
        }

        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_dotnet_backend_task_graph_construction() {
        let dummy_toolchain = DotnetToolchain {
            executable: "dotnet".to_string(),
            dotnet_version: "8.0.0".to_string(),
            csharp_executable: Some("csc".to_string()),
            fsharp_executable: None,
        };

        let backend = DotnetBackend::with_toolchain(dummy_toolchain);

        let config = DotnetProjectConfig {
            project_name: "MyApp".to_string(),
            target_framework: DotnetTargetFramework::Net8_0,
            release: false,
            run_tests: true,
            publish: false,
            output_path: None,
            runtime: None,
        };

        let temp = tempdir().unwrap();
        let graph = backend
            .create_tasks_from_config(&config, temp.path(), &temp.path().join("build"))
            .unwrap();

        assert!(graph.len() >= 3);
        assert_eq!(backend.name(), "dotnet");

        let topo = graph.topological_order();
        assert!(topo.len() >= 3);

        let first = graph.node(topo[0]).unwrap();
        assert!(first.payload.label.contains("restore"));
    }
}
