#![forbid(unsafe_code)]

use std::path::Path;
use thiserror::Error;

use forge_core::{BuildBackend, FingerprintUtils};
use forge_executor::{CacheEntry, CommandSpec, Task};
use forge_graph::BuildGraph;

pub mod config;
pub mod fingerprint;
pub mod toolchain;

pub use config::{DartProjectConfig, DartTargetPlatform};
pub use toolchain::{DartCompiler, DartToolchain};

#[derive(Debug, Error)]
pub enum DartBackendError {
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
pub struct DartBackend {
    pub toolchain: DartToolchain,
}

impl BuildBackend for DartBackend {
    fn name(&self) -> &'static str {
        "dart"
    }
}

impl DartBackend {
    pub fn new() -> Result<Self, DartBackendError> {
        let toolchain =
            DartToolchain::detect().map_err(|e| DartBackendError::Toolchain(e.to_string()))?;
        Ok(Self { toolchain })
    }

    pub fn with_toolchain(toolchain: DartToolchain) -> Self {
        Self { toolchain }
    }

    pub fn create_tasks_from_config(
        &self,
        config: &DartProjectConfig,
        project_dir: &Path,
        output_dir: &Path,
    ) -> Result<BuildGraph<Task>, DartBackendError> {
        let mut graph = BuildGraph::new();
        std::fs::create_dir_all(output_dir)?;

        let namespace = FingerprintUtils::compute_namespace(project_dir);

        let fp = fingerprint::compute_dart_fingerprint(
            project_dir,
            &self.toolchain.dart_version,
            &self.toolchain.flutter_version,
            &config.project_type,
        )
        .unwrap_or_else(|_| "no_fp".to_string());

        if config.is_flutter {
            self.create_flutter_tasks(
                config,
                project_dir,
                output_dir,
                &namespace,
                &fp,
                &mut graph,
            )?;
        } else {
            self.create_dart_tasks(config, project_dir, output_dir, &namespace, &fp, &mut graph)?;
        }

        Ok(graph)
    }

    fn create_dart_tasks(
        &self,
        config: &DartProjectConfig,
        project_dir: &Path,
        _output_dir: &Path,
        namespace: &str,
        fp: &str,
        graph: &mut BuildGraph<Task>,
    ) -> Result<(), DartBackendError> {
        let dart = self
            .toolchain
            .dart_executable
            .as_ref()
            .ok_or_else(|| DartBackendError::Toolchain("Dart not found".to_string()))?;

        let get_args = vec!["pub".to_string(), "get".to_string()];
        let get_spec = CommandSpec::new(dart).args(get_args).cwd(project_dir);
        let get_cache = CacheEntry {
            key: FingerprintUtils::format_cache_key(
                "dart/pub",
                namespace,
                "get",
                &config.project_name,
            ),
            fingerprint: fp.to_string(),
        };
        let get_task = Task::new(
            format!("dart pub get {}", config.project_name),
            get_spec.command_line(),
            get_spec,
        )
        .with_cache(get_cache);
        let get_node_id = graph.add_node(get_task);

        let analyze_args = vec!["analyze".to_string()];
        let analyze_spec = CommandSpec::new(dart).args(analyze_args).cwd(project_dir);
        let analyze_task = Task::new(
            format!("dart analyze {}", config.project_name),
            analyze_spec.command_line(),
            analyze_spec,
        );
        let analyze_node_id = graph.add_node(analyze_task);
        graph.add_dependency(get_node_id, analyze_node_id)?;

        if config.run_tests {
            let test_args = vec!["test".to_string()];
            let test_spec = CommandSpec::new(dart).args(test_args).cwd(project_dir);
            let test_cache = CacheEntry {
                key: FingerprintUtils::format_cache_key(
                    "dart",
                    namespace,
                    "test",
                    &config.project_name,
                ),
                fingerprint: fp.to_string(),
            };
            let test_task = Task::new(
                format!("dart test {}", config.project_name),
                test_spec.command_line(),
                test_spec,
            )
            .with_cache(test_cache);
            let test_node_id = graph.add_node(test_task);
            graph.add_dependency(analyze_node_id, test_node_id)?;
        }

        if config.compile {
            let compile_args = vec!["compile".to_string(), "exe".to_string()];
            let compile_spec = CommandSpec::new(dart).args(compile_args).cwd(project_dir);
            let compile_cache = CacheEntry {
                key: FingerprintUtils::format_cache_key(
                    "dart",
                    namespace,
                    "compile",
                    &config.project_name,
                ),
                fingerprint: fp.to_string(),
            };
            let compile_task = Task::new(
                format!("dart compile exe {}", config.project_name),
                compile_spec.command_line(),
                compile_spec,
            )
            .with_cache(compile_cache);
            let compile_node_id = graph.add_node(compile_task);
            graph.add_dependency(analyze_node_id, compile_node_id)?;
        }

        Ok(())
    }

    fn create_flutter_tasks(
        &self,
        config: &DartProjectConfig,
        project_dir: &Path,
        _output_dir: &Path,
        namespace: &str,
        fp: &str,
        graph: &mut BuildGraph<Task>,
    ) -> Result<(), DartBackendError> {
        let flutter = self
            .toolchain
            .flutter_executable
            .as_ref()
            .ok_or_else(|| DartBackendError::Toolchain("Flutter not found".to_string()))?;

        let pub_get_args = vec!["pub".to_string(), "get".to_string()];
        let pub_get_spec = CommandSpec::new(flutter)
            .args(pub_get_args)
            .cwd(project_dir);
        let pub_get_cache = CacheEntry {
            key: FingerprintUtils::format_cache_key(
                "flutter/pub",
                namespace,
                "get",
                &config.project_name,
            ),
            fingerprint: fp.to_string(),
        };
        let pub_get_task = Task::new(
            format!("flutter pub get {}", config.project_name),
            pub_get_spec.command_line(),
            pub_get_spec,
        )
        .with_cache(pub_get_cache);
        let pub_get_node_id = graph.add_node(pub_get_task);

        let analyze_args = vec!["analyze".to_string()];
        let analyze_spec = CommandSpec::new(flutter)
            .args(analyze_args)
            .cwd(project_dir);
        let analyze_task = Task::new(
            format!("flutter analyze {}", config.project_name),
            analyze_spec.command_line(),
            analyze_spec,
        );
        let analyze_node_id = graph.add_node(analyze_task);
        graph.add_dependency(pub_get_node_id, analyze_node_id)?;

        if config.run_tests {
            let test_args = vec!["test".to_string()];
            let test_spec = CommandSpec::new(flutter).args(test_args).cwd(project_dir);
            let test_cache = CacheEntry {
                key: FingerprintUtils::format_cache_key(
                    "flutter",
                    namespace,
                    "test",
                    &config.project_name,
                ),
                fingerprint: fp.to_string(),
            };
            let test_task = Task::new(
                format!("flutter test {}", config.project_name),
                test_spec.command_line(),
                test_spec,
            )
            .with_cache(test_cache);
            let test_node_id = graph.add_node(test_task);
            graph.add_dependency(analyze_node_id, test_node_id)?;
        }

        let mut build_args = vec!["build".to_string()];
        match &config.target_platform {
            DartTargetPlatform::APK => {
                build_args.push("apk".to_string());
            }
            DartTargetPlatform::IOS => {
                build_args.push("ios".to_string());
            }
            DartTargetPlatform::Web => {
                build_args.push("web".to_string());
            }
            DartTargetPlatform::Windows => {
                build_args.push("windows".to_string());
            }
            DartTargetPlatform::MacOS => {
                build_args.push("macos".to_string());
            }
            DartTargetPlatform::Linux => {
                build_args.push("linux".to_string());
            }
            DartTargetPlatform::All => {
                build_args.push("apk".to_string());
            }
        }

        if config.release {
            build_args.push("--release".to_string());
        }

        let build_spec = CommandSpec::new(flutter).args(build_args).cwd(project_dir);
        let build_cache = CacheEntry {
            key: FingerprintUtils::format_cache_key(
                "flutter",
                namespace,
                "build",
                &config.project_name,
            ),
            fingerprint: fp.to_string(),
        };
        let build_task = Task::new(
            format!("flutter build {}", config.project_name),
            build_spec.command_line(),
            build_spec,
        )
        .with_cache(build_cache);
        let build_node_id = graph.add_node(build_task);
        graph.add_dependency(analyze_node_id, build_node_id)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_dart_backend_task_graph_construction() {
        let dummy_toolchain = DartToolchain {
            dart_executable: Some("dart".to_string()),
            dart_version: "3.0.0".to_string(),
            flutter_executable: Some("flutter".to_string()),
            flutter_version: Some("3.10.0".to_string()),
        };

        let backend = DartBackend::with_toolchain(dummy_toolchain);

        let config = DartProjectConfig {
            project_name: "my_dart_app".to_string(),
            project_type: crate::config::DartProjectType::Flutter,
            target_platform: DartTargetPlatform::APK,
            release: false,
            run_tests: true,
            compile: false,
            is_flutter: true,
        };

        let temp = tempdir().unwrap();
        let graph = backend
            .create_tasks_from_config(&config, temp.path(), &temp.path().join("build"))
            .unwrap();

        assert!(graph.len() >= 4);
        assert_eq!(backend.name(), "dart");

        let topo = graph.topological_order();
        assert!(topo.len() >= 4);

        let first = graph.node(topo[0]).unwrap();
        assert!(first.payload.label.contains("pub get"));
    }
}
