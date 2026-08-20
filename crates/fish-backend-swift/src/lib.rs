#![forbid(unsafe_code)]

use std::path::Path;
use thiserror::Error;

use fish_core::{BuildBackend, FingerprintUtils};
use fish_executor::{CacheEntry, CommandSpec, Task};
use fish_graph::BuildGraph;

pub mod config;
pub mod fingerprint;
pub mod toolchain;

pub use config::{SwiftPlatform, SwiftProjectConfig};
pub use toolchain::{SwiftCompiler, SwiftToolchain};

#[derive(Debug, Error)]
pub enum SwiftBackendError {
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
pub struct SwiftBackend {
    pub toolchain: SwiftToolchain,
}

impl BuildBackend for SwiftBackend {
    fn name(&self) -> &'static str {
        "swift"
    }
}

impl SwiftBackend {
    pub fn new() -> Result<Self, SwiftBackendError> {
        let toolchain =
            SwiftToolchain::detect().map_err(|e| SwiftBackendError::Toolchain(e.to_string()))?;
        Ok(Self { toolchain })
    }

    pub fn with_toolchain(toolchain: SwiftToolchain) -> Self {
        Self { toolchain }
    }

    pub fn create_tasks_from_config(
        &self,
        config: &SwiftProjectConfig,
        project_dir: &Path,
        output_dir: &Path,
    ) -> Result<BuildGraph<Task>, SwiftBackendError> {
        let mut graph = BuildGraph::new();
        std::fs::create_dir_all(output_dir)?;

        let namespace = FingerprintUtils::compute_namespace(project_dir);

        let fp = fingerprint::compute_swift_fingerprint(
            project_dir,
            &self.toolchain.swift_version,
            &config.platform,
        )
        .unwrap_or_else(|_| "no_fp".to_string());

        let clean_args = vec!["package".to_string(), "clean".to_string()];
        let clean_spec = CommandSpec::new(&self.toolchain.executable)
            .args(clean_args)
            .cwd(project_dir);
        let clean_task = Task::new(
            format!("swift package clean {}", config.package_name),
            clean_spec.command_line(),
            clean_spec,
        );
        let clean_node_id = graph.add_node(clean_task);

        let resolve_args = vec!["package".to_string(), "resolve".to_string()];
        let resolve_spec = CommandSpec::new(&self.toolchain.executable)
            .args(resolve_args)
            .cwd(project_dir);
        let resolve_cache = CacheEntry {
            key: FingerprintUtils::format_cache_key(
                "swift",
                &namespace,
                "resolve",
                &config.package_name,
            ),
            fingerprint: fp.clone(),
        };
        let resolve_task = Task::new(
            format!("swift package resolve {}", config.package_name),
            resolve_spec.command_line(),
            resolve_spec,
        )
        .with_cache(resolve_cache);
        let resolve_node_id = graph.add_node(resolve_task);
        graph.add_dependency(clean_node_id, resolve_node_id)?;

        let mut build_args = vec!["build".to_string()];
        if config.release {
            build_args.push("-c".to_string());
            build_args.push("release".to_string());
        } else {
            build_args.push("-c".to_string());
            build_args.push("debug".to_string());
        }

        match &config.platform {
            SwiftPlatform::IOS => {
                build_args.push("--triple".to_string());
                build_args.push("arm64-apple-ios".to_string());
            }
            SwiftPlatform::MacOS => {
                build_args.push("--triple".to_string());
                build_args.push("x86_64-apple-macosx".to_string());
            }
            SwiftPlatform::TVOS => {
                build_args.push("--triple".to_string());
                build_args.push("arm64-apple-tvos".to_string());
            }
            SwiftPlatform::WatchOS => {
                build_args.push("--triple".to_string());
                build_args.push("arm64-apple-watchos".to_string());
            }
            SwiftPlatform::Linux => {
                build_args.push("--triple".to_string());
                build_args.push("x86_64-unknown-linux".to_string());
            }
        }

        let build_spec = CommandSpec::new(&self.toolchain.executable)
            .args(build_args)
            .cwd(project_dir);
        let build_cache = CacheEntry {
            key: FingerprintUtils::format_cache_key(
                "swift",
                &namespace,
                "build",
                &config.package_name,
            ),
            fingerprint: fp.clone(),
        };
        let build_task = Task::new(
            format!("swift build {}", config.package_name),
            build_spec.command_line(),
            build_spec,
        )
        .with_cache(build_cache);
        let build_node_id = graph.add_node(build_task);
        graph.add_dependency(resolve_node_id, build_node_id)?;

        if config.run_tests {
            let mut test_args = vec!["test".to_string()];
            if config.release {
                test_args.push("-c".to_string());
                test_args.push("release".to_string());
            }

            let test_spec = CommandSpec::new(&self.toolchain.executable)
                .args(test_args)
                .cwd(project_dir);
            let test_cache = CacheEntry {
                key: FingerprintUtils::format_cache_key(
                    "swift",
                    &namespace,
                    "test",
                    &config.package_name,
                ),
                fingerprint: fp,
            };
            let test_task = Task::new(
                format!("swift test {}", config.package_name),
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
    fn test_swift_backend_task_graph_construction() {
        let dummy_toolchain = SwiftToolchain {
            executable: "swift".to_string(),
            swift_version: "5.9.0".to_string(),
            clang_executable: Some("clang".to_string()),
            clang_version: None,
        };

        let backend = SwiftBackend::with_toolchain(dummy_toolchain);

        let config = SwiftProjectConfig {
            package_name: "MySwiftApp".to_string(),
            platform: SwiftPlatform::MacOS,
            release: false,
            run_tests: true,
        };

        let temp = tempdir().unwrap();
        let graph = backend
            .create_tasks_from_config(&config, temp.path(), &temp.path().join("build"))
            .unwrap();

        assert!(graph.len() >= 4);
        assert_eq!(backend.name(), "swift");

        let topo = graph.topological_order();
        assert!(topo.len() >= 4);

        let first = graph.node(topo[0]).unwrap();
        assert!(first.payload.label.contains("clean"));
    }
}
