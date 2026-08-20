#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;

use fish_backend_cc::{CcBackend, CcLanguage, CcOutputType, CcProjectConfig};
use fish_backend_dart::{DartBackend, DartProjectConfig};
use fish_backend_docker::DockerBackend;
use fish_backend_dotnet::{DotnetBackend, DotnetProjectConfig};
use fish_backend_go::{GoBackend, GoProjectConfig};
use fish_backend_java::{JavaBackend, JavaProjectConfig};
use fish_backend_py::{PyBackend, PyProjectConfig};
use fish_backend_rust::{BuildMode, RustBackend};
use fish_backend_swift::{SwiftBackend, SwiftProjectConfig};
use fish_backend_ts::{TsBackend, TsProjectConfig};
use fish_backend_zig::{ZigBackend, ZigProjectConfig};
use fish_core::project::Project;
use fish_executor::Task;
use fish_graph::{BuildGraph, NodeId};
use fish_incremental::ecosystem::{EcosystemInfo, EcosystemType, detect_ecosystems};

pub struct PolyglotGraphBuilder;

impl PolyglotGraphBuilder {
    pub fn build_unified_graph(
        start_dir: &Path,
        mode: BuildMode,
    ) -> Result<BuildGraph<Task>, anyhow::Error> {
        let ecosystems = detect_ecosystems(start_dir);
        if ecosystems.is_empty() {
            if let Ok(Some(project)) = Project::discover(start_dir) {
                let backend = RustBackend::new()?;
                let package_graph = if mode == BuildMode::Test {
                    project.build_test_graph()?
                } else {
                    project.build_graph()?
                };
                return backend
                    .create_tasks(&project, &package_graph, mode, true)
                    .map_err(Into::into);
            }
            return Err(anyhow::anyhow!(
                "No supported build ecosystem detected in `{}`",
                start_dir.display()
            ));
        }

        let mut master_graph = BuildGraph::new();
        let mut ecosystem_node_map: HashMap<EcosystemType, Vec<NodeId>> = HashMap::new();
        let mut processed_rust = false;

        for info in &ecosystems {
            if info.ecosystem == EcosystemType::Rust {
                if processed_rust {
                    continue;
                }
                processed_rust = true;
            }
            let manifest_dir =
                if info.ecosystem == EcosystemType::Rust && start_dir.join("Cargo.toml").exists() {
                    start_dir
                } else {
                    info.manifest_path.parent().unwrap_or(start_dir)
                };
            let sub_graph_res = Self::build_subgraph_for_ecosystem(info, manifest_dir, mode);
            if let Ok(sub_graph) = sub_graph_res
                && !sub_graph.is_empty()
            {
                let id_map = master_graph.merge_subgraph(sub_graph);
                let new_ids: Vec<NodeId> = id_map.into_values().collect();
                ecosystem_node_map
                    .entry(info.ecosystem)
                    .or_default()
                    .extend(new_ids);
            }
        }

        if let Some(docker_nodes) = ecosystem_node_map.get(&EcosystemType::Docker) {
            for (eco_type, nodes) in &ecosystem_node_map {
                if *eco_type != EcosystemType::Docker {
                    for &dep in nodes {
                        for &docker_node in docker_nodes {
                            let _ = master_graph.add_dependency(dep, docker_node);
                        }
                    }
                }
            }
        }

        if master_graph.is_empty()
            && let Ok(Some(project)) = Project::discover(start_dir)
        {
            let backend = RustBackend::new()?;
            let package_graph = if mode == BuildMode::Test {
                project.build_test_graph()?
            } else {
                project.build_graph()?
            };
            return backend
                .create_tasks(&project, &package_graph, mode, true)
                .map_err(Into::into);
        }

        Ok(master_graph)
    }

    fn build_subgraph_for_ecosystem(
        info: &EcosystemInfo,
        dir: &Path,
        mode: BuildMode,
    ) -> Result<BuildGraph<Task>, anyhow::Error> {
        match info.ecosystem {
            EcosystemType::Rust => {
                if let Ok(Some(project)) = Project::discover(dir) {
                    let backend = RustBackend::new()?;
                    let package_graph = if mode == BuildMode::Test {
                        project.build_test_graph()?
                    } else {
                        project.build_graph()?
                    };
                    backend
                        .create_tasks(&project, &package_graph, mode, true)
                        .map_err(Into::into)
                } else {
                    Ok(BuildGraph::new())
                }
            }
            EcosystemType::TypeScript => {
                let config = TsProjectConfig::discover_or_default(dir)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let backend = TsBackend::new();
                backend
                    .build_task_graph(&config, dir)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
            }
            EcosystemType::Go => {
                let config_path = dir.join("forge.go.json");
                let config = if config_path.exists() {
                    GoProjectConfig::from_file(&config_path)?
                } else {
                    let name = dir
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("app")
                        .to_string();
                    GoProjectConfig {
                        name,
                        package_path: "./...".to_string(),
                        tags: vec![],
                        ldflags: None,
                        gcflags: None,
                        run_tests: true,
                        output_binary: None,
                    }
                };
                let backend = GoBackend::new()?;
                let build_dir = dir.join("build");
                backend
                    .create_tasks_from_config(&config, dir, &build_dir)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
            }
            EcosystemType::Python => {
                let config = PyProjectConfig::discover_or_default(dir)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let backend = PyBackend::new();
                backend
                    .build_task_graph(&config, dir)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
            }
            EcosystemType::Cpp => {
                let config_path = dir.join("fish.cc.json");
                let config = if config_path.exists() {
                    CcProjectConfig::from_file(&config_path)?
                } else {
                    let name = dir
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("app")
                        .to_string();
                    CcProjectConfig {
                        name,
                        language: CcLanguage::Cpp,
                        sources: vec![],
                        includes: vec!["include".to_string()],
                        cflags: vec![],
                        cxxflags: vec![],
                        ldflags: vec![],
                        output_type: CcOutputType::Executable,
                    }
                };
                let backend = CcBackend::new(config.language)?;
                let build_dir = dir.join("build");
                backend
                    .create_tasks_from_config(&config, dir, &build_dir)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
            }
            EcosystemType::Java => {
                let config =
                    JavaProjectConfig::detect(dir).map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let backend = JavaBackend::new()?;
                let build_dir = dir.join("build");
                backend
                    .create_tasks_from_config(&config, dir, &build_dir)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
            }
            EcosystemType::DotNet => {
                let config =
                    DotnetProjectConfig::detect(dir).map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let backend = DotnetBackend::new()?;
                let build_dir = dir.join("build");
                backend
                    .create_tasks_from_config(&config, dir, &build_dir)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
            }
            EcosystemType::Swift => {
                let config =
                    SwiftProjectConfig::detect(dir).map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let backend = SwiftBackend::new()?;
                let build_dir = dir.join("build");
                backend
                    .create_tasks_from_config(&config, dir, &build_dir)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
            }
            EcosystemType::Dart => {
                let config =
                    DartProjectConfig::detect(dir).map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let backend = DartBackend::new()?;
                let build_dir = dir.join("build");
                backend
                    .create_tasks_from_config(&config, dir, &build_dir)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
            }
            EcosystemType::Zig => {
                let config =
                    ZigProjectConfig::detect(dir).map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let backend = ZigBackend::new()?;
                let build_dir = dir.join("build");
                backend
                    .create_tasks_from_config(&config, dir, &build_dir)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))
            }
            EcosystemType::Docker => {
                if let Some(config) = DockerBackend::detect_config(dir) {
                    let backend = DockerBackend::new(config)?;
                    backend
                        .build_task_graph()
                        .map_err(|e| anyhow::anyhow!(e.to_string()))
                } else {
                    Ok(BuildGraph::new())
                }
            }
            _ => Ok(BuildGraph::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_polyglot_graph_builder_empty_dir() {
        let dir = tempdir().unwrap();
        let res = PolyglotGraphBuilder::build_unified_graph(dir.path(), BuildMode::Build);
        assert!(res.is_err());
    }

    #[test]
    fn test_polyglot_graph_builder_rust_project() {
        let dir = tempdir().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "test_rust_pkg"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();
        let src_dir = dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("lib.rs"), "").unwrap();

        let graph =
            PolyglotGraphBuilder::build_unified_graph(dir.path(), BuildMode::Build).unwrap();
        assert_eq!(graph.len(), 1);
    }
}
