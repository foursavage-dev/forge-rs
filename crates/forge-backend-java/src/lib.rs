#![forbid(unsafe_code)]

use std::path::Path;
use thiserror::Error;

use forge_core::{BuildBackend, FingerprintUtils};
use forge_executor::{CacheEntry, CommandSpec, Task};
use forge_graph::BuildGraph;

pub mod config;
pub mod fingerprint;
pub mod toolchain;

pub use config::{JavaBuildSystem, JavaProjectConfig};
pub use toolchain::{JavaCompiler, JavaToolchain};

#[derive(Debug, Error)]
pub enum JavaBackendError {
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
pub struct JavaBackend {
    pub toolchain: JavaToolchain,
}

impl BuildBackend for JavaBackend {
    fn name(&self) -> &'static str {
        "java"
    }
}

impl JavaBackend {
    pub fn new() -> Result<Self, JavaBackendError> {
        let toolchain =
            JavaToolchain::detect().map_err(|e| JavaBackendError::Toolchain(e.to_string()))?;
        Ok(Self { toolchain })
    }

    pub fn with_toolchain(toolchain: JavaToolchain) -> Self {
        Self { toolchain }
    }

    pub fn create_tasks_from_config(
        &self,
        config: &JavaProjectConfig,
        project_dir: &Path,
        output_dir: &Path,
    ) -> Result<BuildGraph<Task>, JavaBackendError> {
        let mut graph = BuildGraph::new();
        std::fs::create_dir_all(output_dir)?;
        let namespace = FingerprintUtils::compute_namespace(project_dir);

        let fp = fingerprint::compute_java_fingerprint(
            project_dir,
            &self.toolchain.java_version,
            &self.toolchain.kotlin_version,
            &config.build_system,
        )
        .unwrap_or_else(|_| "no_fp".to_string());

        match &config.build_system {
            JavaBuildSystem::Maven => {
                self.create_maven_tasks(
                    config,
                    project_dir,
                    output_dir,
                    &namespace,
                    &fp,
                    &mut graph,
                )?;
            }
            JavaBuildSystem::Gradle => {
                self.create_gradle_tasks(
                    config,
                    project_dir,
                    output_dir,
                    &namespace,
                    &fp,
                    &mut graph,
                )?;
            }
        }

        Ok(graph)
    }

    fn create_maven_tasks(
        &self,
        config: &JavaProjectConfig,
        project_dir: &Path,
        _output_dir: &Path,
        namespace: &str,
        fp: &str,
        graph: &mut BuildGraph<Task>,
    ) -> Result<(), JavaBackendError> {
        let maven = self
            .toolchain
            .maven_executable
            .as_ref()
            .ok_or_else(|| JavaBackendError::Toolchain("Maven not found".to_string()))?;

        let clean_args = vec!["clean".to_string()];
        let clean_spec = CommandSpec::new(maven).args(clean_args).cwd(project_dir);
        let clean_task = Task::new(
            format!("mvn clean {}", config.group_id),
            clean_spec.command_line(),
            clean_spec,
        );
        let clean_node_id = graph.add_node(clean_task);

        let mut compile_args = vec!["compile".to_string()];
        if config.skip_tests {
            compile_args.push("-DskipTests".to_string());
        }
        let compile_spec = CommandSpec::new(maven).args(compile_args).cwd(project_dir);
        let compile_cache = CacheEntry {
            key: FingerprintUtils::format_cache_key(
                "java/maven",
                namespace,
                "compile",
                &config.artifact_id,
            ),
            fingerprint: fp.to_string(),
        };
        let compile_task = Task::new(
            format!("mvn compile {}", config.artifact_id),
            compile_spec.command_line(),
            compile_spec,
        )
        .with_cache(compile_cache);
        let compile_node_id = graph.add_node(compile_task);
        graph.add_dependency(clean_node_id, compile_node_id)?;

        let mut package_args = vec!["package".to_string()];
        if config.skip_tests {
            package_args.push("-DskipTests".to_string());
        }
        let package_spec = CommandSpec::new(maven).args(package_args).cwd(project_dir);
        let package_cache = CacheEntry {
            key: FingerprintUtils::format_cache_key(
                "java/maven",
                namespace,
                "package",
                &config.artifact_id,
            ),
            fingerprint: fp.to_string(),
        };
        let package_task = Task::new(
            format!("mvn package {}", config.artifact_id),
            package_spec.command_line(),
            package_spec,
        )
        .with_cache(package_cache);
        let package_node_id = graph.add_node(package_task);
        graph.add_dependency(compile_node_id, package_node_id)?;

        if !config.skip_tests {
            let test_args = vec!["test".to_string()];
            let test_spec = CommandSpec::new(maven).args(test_args).cwd(project_dir);
            let test_cache = CacheEntry {
                key: FingerprintUtils::format_cache_key(
                    "java/maven",
                    namespace,
                    "test",
                    &config.artifact_id,
                ),
                fingerprint: fp.to_string(),
            };
            let test_task = Task::new(
                format!("mvn test {}", config.artifact_id),
                test_spec.command_line(),
                test_spec,
            )
            .with_cache(test_cache);
            let test_node_id = graph.add_node(test_task);
            graph.add_dependency(compile_node_id, test_node_id)?;
        }

        Ok(())
    }

    fn create_gradle_tasks(
        &self,
        config: &JavaProjectConfig,
        project_dir: &Path,
        _output_dir: &Path,
        namespace: &str,
        fp: &str,
        graph: &mut BuildGraph<Task>,
    ) -> Result<(), JavaBackendError> {
        let gradle = self
            .toolchain
            .gradle_executable
            .as_ref()
            .ok_or_else(|| JavaBackendError::Toolchain("Gradle not found".to_string()))?;

        let clean_args = vec!["clean".to_string()];
        let clean_spec = CommandSpec::new(gradle).args(clean_args).cwd(project_dir);
        let clean_task = Task::new(
            format!("gradle clean {}", config.group_id),
            clean_spec.command_line(),
            clean_spec,
        );
        let clean_node_id = graph.add_node(clean_task);

        let mut build_args = vec!["build".to_string()];
        if config.skip_tests {
            build_args.push("-x".to_string());
            build_args.push("test".to_string());
        }
        let build_spec = CommandSpec::new(gradle).args(build_args).cwd(project_dir);
        let build_cache = CacheEntry {
            key: FingerprintUtils::format_cache_key(
                "java/gradle",
                namespace,
                "build",
                &config.artifact_id,
            ),
            fingerprint: fp.to_string(),
        };
        let build_task = Task::new(
            format!("gradle build {}", config.artifact_id),
            build_spec.command_line(),
            build_spec,
        )
        .with_cache(build_cache);
        let build_node_id = graph.add_node(build_task);
        graph.add_dependency(clean_node_id, build_node_id)?;

        if !config.skip_tests {
            let test_args = vec!["test".to_string()];
            let test_spec = CommandSpec::new(gradle).args(test_args).cwd(project_dir);
            let test_cache = CacheEntry {
                key: FingerprintUtils::format_cache_key(
                    "java/gradle",
                    namespace,
                    "test",
                    &config.artifact_id,
                ),
                fingerprint: fp.to_string(),
            };
            let test_task = Task::new(
                format!("gradle test {}", config.artifact_id),
                test_spec.command_line(),
                test_spec,
            )
            .with_cache(test_cache);
            let test_node_id = graph.add_node(test_task);
            graph.add_dependency(build_node_id, test_node_id)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_java_backend_task_graph_construction() {
        let dummy_toolchain = JavaToolchain {
            java_executable: "java".to_string(),
            java_version: "openjdk version \"17.0.2\"".to_string(),
            javac_executable: Some("javac".to_string()),
            kotlin_executable: None,
            kotlin_version: None,
            maven_executable: Some("mvn".to_string()),
            gradle_executable: Some("gradle".to_string()),
        };

        let backend = JavaBackend::with_toolchain(dummy_toolchain);

        let config = JavaProjectConfig {
            group_id: "com.example".to_string(),
            artifact_id: "my-app".to_string(),
            version: "1.0.0".to_string(),
            build_system: JavaBuildSystem::Maven,
            skip_tests: false,
        };

        let temp = tempdir().unwrap();
        let graph = backend
            .create_tasks_from_config(&config, temp.path(), &temp.path().join("build"))
            .unwrap();

        assert!(graph.len() >= 3);
        assert_eq!(backend.name(), "java");

        let topo = graph.topological_order();
        assert!(topo.len() >= 3);

        let first = graph.node(topo[0]).unwrap();
        assert!(first.payload.label.contains("clean"));
    }
}
