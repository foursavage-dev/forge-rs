#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

use cargo_metadata::PackageId;
use fish_core::project::Project;
use fish_core::{BinaryUtils, BuildBackend, FingerprintUtils, ToolchainUtils};
use fish_executor::{CacheEntry, CommandSpec, Task};
use fish_graph::{BuildGraph, NodeId};

pub mod fingerprint;
pub mod rustc;

pub use rustc::RustcCompiler;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    Build,
    Check,
    Test,
}

impl BuildMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Check => "check",
            Self::Test => "test",
        }
    }

    pub fn cargo_subcommand(&self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Check => "check",
            Self::Test => "test",
        }
    }
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("toolchain error: {0}")]
    Toolchain(String),
    #[error("graph error: {0}")]
    Graph(#[from] fish_graph::GraphError),
    #[error("failed to read `{}`: {source}", path.display())]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{0}")]
    Message(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct RustBackend {
    pub toolchain: String,
    pub rustc: RustcCompiler,
}

impl BuildBackend for RustBackend {
    fn name(&self) -> &'static str {
        "rust"
    }
}

impl RustBackend {
    pub fn new() -> Result<Self, BackendError> {
        let toolchain = ToolchainUtils::get_tool_version("rustc", &["--version"])
            .map_err(BackendError::Toolchain)?;
        let rustc = RustcCompiler::detect().map_err(BackendError::Toolchain)?;
        Ok(Self { toolchain, rustc })
    }

    pub fn create_tasks(
        &self,
        project: &Project,
        package_graph: &BuildGraph<PackageId>,
        mode: BuildMode,
        caching: bool,
    ) -> Result<BuildGraph<Task>, BackendError> {
        let workspace_root = PathBuf::from(project.workspace_root().as_str());
        let lock_file = workspace_root.join("Cargo.lock");
        let lock_file = lock_file.is_file().then_some(lock_file);
        let namespace = FingerprintUtils::compute_namespace(&workspace_root);

        let mut fingerprints: HashMap<NodeId, String> = HashMap::new();
        for level in package_graph.levels() {
            for id in level {
                let package = package_graph
                    .node(id)
                    .expect("levels contain existing nodes")
                    .payload
                    .clone();
                let metadata = project.package(&package).ok_or_else(|| {
                    BackendError::Message(format!(
                        "package `{package}` was not found in cargo metadata"
                    ))
                })?;
                let package_dir = metadata
                    .manifest_path
                    .parent()
                    .map(|parent| PathBuf::from(parent.as_str()))
                    .ok_or_else(|| {
                        BackendError::Message(format!(
                            "package `{package}` has no manifest directory"
                        ))
                    })?;
                let own = fingerprint::package_input_fingerprint(
                    &package_dir,
                    lock_file.as_deref(),
                    &self.toolchain,
                    mode,
                )?;
                let dep_fingerprints: Vec<String> = package_graph
                    .deps(id)?
                    .iter()
                    .filter_map(|dep| fingerprints.get(dep).cloned())
                    .collect();
                fingerprints.insert(
                    id,
                    FingerprintUtils::combine_fingerprints(&own, &dep_fingerprints),
                );
            }
        }

        let out_dir = workspace_root.join("target").join("fish_artifacts");
        let _ = std::fs::create_dir_all(&out_dir);

        let cargo = std::env::var_os("CARGO")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("cargo"));
        let cargo = cargo.to_string_lossy().into_owned();

        let profile = "debug";

        let levels = package_graph.levels();
        let mut level_node: HashMap<NodeId, NodeId> = HashMap::new();
        let mut task_graph = BuildGraph::new();

        for level in &levels {
            let mut members: Vec<NodeId> = level.to_vec();
            members.sort_by_key(|id| {
                let package_id = &package_graph
                    .node(*id)
                    .expect("level members are package graph nodes")
                    .payload;
                project
                    .package(package_id)
                    .expect("level members are metadata packages")
                    .name
                    .to_string()
            });

            let names: Vec<String> = members
                .iter()
                .map(|id| {
                    let package_id = &package_graph
                        .node(*id)
                        .expect("level members are package graph nodes")
                        .payload;
                    project
                        .package(package_id)
                        .expect("level members are metadata packages")
                        .name
                        .to_string()
                })
                .collect();

            let label = if names.len() == 1 {
                names[0].clone()
            } else {
                names.join(", ")
            };

            let mut args = vec![mode.cargo_subcommand().to_string()];
            for name in &names {
                args.push("--package".to_string());
                args.push(name.clone());
            }
            let spec = CommandSpec::new(&cargo).args(args).cwd(&workspace_root);
            let description = spec.command_line();

            let artifacts = if mode == BuildMode::Build {
                bin_outputs(project, &members, package_graph, &workspace_root, profile)
            } else {
                Vec::new()
            };

            let mut task = Task::new(label, description, spec).with_artifacts(artifacts);
            if caching {
                let fingerprint = FingerprintUtils::combine_fingerprints(
                    "",
                    &members
                        .iter()
                        .filter_map(|id| fingerprints.get(id).cloned())
                        .collect::<Vec<_>>(),
                );
                task = task.with_cache(CacheEntry {
                    key: FingerprintUtils::format_cache_key(
                        "v1",
                        &namespace,
                        &format!("{}/level", mode.as_str()),
                        &names.join("+"),
                    ),
                    fingerprint,
                });
            }

            let node_id = task_graph.add_node(task);
            for id in level {
                level_node.insert(*id, node_id);
            }
        }

        for level in &levels {
            let node = level_node
                .get(&level[0])
                .expect("every package maps to a level task");
            for id in level {
                for dep in package_graph.deps(*id)? {
                    let dep_node = level_node
                        .get(dep)
                        .expect("dependencies live in earlier levels");
                    if dep_node != node {
                        task_graph.add_dependency(*dep_node, *node)?;
                    }
                }
            }
        }

        Ok(task_graph)
    }
}

fn bin_outputs(
    project: &Project,
    members: &[NodeId],
    package_graph: &BuildGraph<PackageId>,
    workspace_root: &Path,
    profile: &str,
) -> Vec<PathBuf> {
    let mut outputs = Vec::new();
    for id in members {
        let package_id = &package_graph
            .node(*id)
            .expect("level members are package graph nodes")
            .payload;
        let Some(package) = project.package(package_id) else {
            continue;
        };
        for target in &package.targets {
            if target.kind.iter().any(|kind| kind.to_string() == "bin") {
                let name = BinaryUtils::add_binary_extension(&target.name);
                outputs.push(workspace_root.join("target").join(profile).join(name));
            }
        }
    }
    outputs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_backend_name() {
        let backend = RustBackend::new().expect("backend created");
        assert_eq!(backend.name(), "rust");
    }
}
