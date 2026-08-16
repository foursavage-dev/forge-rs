#![forbid(unsafe_code)]

use std::path::Path;
use thiserror::Error;

use forge_core::{BuildBackend, FingerprintUtils};
use forge_executor::{CacheEntry, CommandSpec, Task};
use forge_graph::BuildGraph;

pub mod config;
pub mod fingerprint;
pub mod toolchain;

pub use config::{ZigProjectConfig, ZigTarget};
pub use toolchain::{ZigCompiler, ZigToolchain};

#[derive(Debug, Error)]
pub enum ZigBackendError {
    #[error("toolchain error: {0}")]
    Toolchain(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("graph error: {0}")]
    Graph(#[from] forge_graph::GraphError),
    #[error("parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone)]
pub struct ZigBackend {
    pub toolchain: ZigToolchain,
}

impl BuildBackend for ZigBackend {
    fn name(&self) -> &'static str {
        "zig"
    }
}

impl ZigBackend {
    pub fn new() -> Result<Self, ZigBackendError> {
        let toolchain =
            ZigToolchain::detect().map_err(|e| ZigBackendError::Toolchain(e.to_string()))?;
        Ok(Self { toolchain })
    }

    pub fn with_toolchain(toolchain: ZigToolchain) -> Self {
        Self { toolchain }
    }

    pub fn create_tasks_from_config(
        &self,
        config: &ZigProjectConfig,
        project_dir: &Path,
        output_dir: &Path,
    ) -> Result<BuildGraph<Task>, ZigBackendError> {
        let mut graph = BuildGraph::new();
        std::fs::create_dir_all(output_dir)?;

        let namespace = FingerprintUtils::compute_namespace(project_dir);

        let fp = fingerprint::compute_zig_fingerprint(
            project_dir,
            &self.toolchain.zig_version,
            &config.target,
        )
        .unwrap_or_else(|_| "no_fp".to_string());

        let fetch_args = vec!["fetch".to_string()];
        let fetch_spec = CommandSpec::new(&self.toolchain.executable)
            .args(fetch_args)
            .cwd(project_dir);
        let fetch_cache = CacheEntry {
            key: FingerprintUtils::format_cache_key(
                "zig",
                &namespace,
                "fetch",
                &config.project_name,
            ),
            fingerprint: fp.clone(),
        };
        let fetch_task = Task::new(
            format!("zig fetch {}", config.project_name),
            fetch_spec.command_line(),
            fetch_spec,
        )
        .with_cache(fetch_cache);
        let fetch_node_id = graph.add_node(fetch_task);

        let mut build_args = vec!["build".to_string()];

        if config.release {
            build_args.push("-Doptimize".to_string());
            build_args.push("ReleaseFast".to_string());
        } else {
            build_args.push("-Doptimize".to_string());
            build_args.push("Debug".to_string());
        }

        match &config.target {
            ZigTarget::Native => {}
            ZigTarget::X86_64Linux => {
                build_args.push("-target".to_string());
                build_args.push("x86_64-linux-gnu".to_string());
            }
            ZigTarget::X86_64Windows => {
                build_args.push("-target".to_string());
                build_args.push("x86_64-windows-gnu".to_string());
            }
            ZigTarget::X86_64MacOS => {
                build_args.push("-target".to_string());
                build_args.push("x86_64-macos-gnu".to_string());
            }
            ZigTarget::Aarch64Linux => {
                build_args.push("-target".to_string());
                build_args.push("aarch64-linux-gnu".to_string());
            }
            ZigTarget::Aarch64MacOS => {
                build_args.push("-target".to_string());
                build_args.push("aarch64-macos-gnu".to_string());
            }
            ZigTarget::Wasm32 => {
                build_args.push("-target".to_string());
                build_args.push("wasm32-wasi".to_string());
            }
            ZigTarget::Custom(triple) => {
                build_args.push("-target".to_string());
                build_args.push(triple.clone());
            }
        }

        let build_spec = CommandSpec::new(&self.toolchain.executable)
            .args(build_args)
            .cwd(project_dir);
        let build_cache = CacheEntry {
            key: FingerprintUtils::format_cache_key(
                "zig",
                &namespace,
                "build",
                &config.project_name,
            ),
            fingerprint: fp.clone(),
        };
        let build_task = Task::new(
            format!("zig build {}", config.project_name),
            build_spec.command_line(),
            build_spec,
        )
        .with_cache(build_cache);
        let build_node_id = graph.add_node(build_task);
        graph.add_dependency(fetch_node_id, build_node_id)?;

        if config.run_tests {
            let test_args = vec!["test".to_string()];
            let test_spec = CommandSpec::new(&self.toolchain.executable)
                .args(test_args)
                .cwd(project_dir);
            let test_cache = CacheEntry {
                key: FingerprintUtils::format_cache_key(
                    "zig",
                    &namespace,
                    "test",
                    &config.project_name,
                ),
                fingerprint: fp,
            };
            let test_task = Task::new(
                format!("zig test {}", config.project_name),
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
    fn test_zig_backend_task_graph_construction() {
        let dummy_toolchain = ZigToolchain {
            executable: "zig".to_string(),
            zig_version: "0.11.0".to_string(),
        };

        let backend = ZigBackend::with_toolchain(dummy_toolchain);

        let config = ZigProjectConfig {
            project_name: "my_zig_app".to_string(),
            target: ZigTarget::Native,
            release: false,
            run_tests: true,
        };

        let temp = tempdir().unwrap();
        let graph = backend
            .create_tasks_from_config(&config, temp.path(), &temp.path().join("build"))
            .unwrap();

        assert!(graph.len() >= 3);
        assert_eq!(backend.name(), "zig");

        let topo = graph.topological_order();
        assert!(topo.len() >= 3);

        let first = graph.node(topo[0]).unwrap();
        assert!(first.payload.label.contains("fetch"));
    }
}
