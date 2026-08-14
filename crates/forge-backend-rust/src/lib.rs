#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

use cargo_metadata::PackageId;
use forge_core::BuildBackend;
use forge_core::project::Project;
use forge_executor::{CacheEntry, CommandSpec, Task};
use forge_graph::{BuildGraph, NodeId};

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
    Graph(#[from] forge_graph::GraphError),
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
        let toolchain = tool_version("rustc", &["--version"])?;
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
        let namespace = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(workspace_root.to_string_lossy().as_bytes());
            hasher.finalize().to_hex().to_string()[..12].to_string()
        };

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
                let mut dep_fingerprints = Vec::new();
                for dep in package_graph.deps(id)? {
                    if let Some(fp) = fingerprints.get(dep) {
                        dep_fingerprints.push(fp.clone());
                    }
                }
                fingerprints.insert(
                    id,
                    fingerprint::combined_fingerprint(&own, &dep_fingerprints),
                );
            }
        }

        let out_dir = workspace_root.join("target").join("forge_artifacts");
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
                let fingerprint = fingerprint::combined_fingerprint(
                    "",
                    &members
                        .iter()
                        .filter_map(|id| fingerprints.get(id).cloned())
                        .collect::<Vec<_>>(),
                );
                task = task.with_cache(CacheEntry {
                    key: format!("v1/{namespace}/{}/level/{}", mode.as_str(), names.join("+")),
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

fn tool_version(tool: &str, args: &[&str]) -> Result<String, BackendError> {
    let output = std::process::Command::new(tool)
        .args(args)
        .output()
        .map_err(|source| BackendError::Toolchain(format!("failed to run `{tool}`: {source}")))?;
    if !output.status.success() {
        return Err(BackendError::Toolchain(format!(
            "`{tool}` exited with {}",
            output.status
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().next().unwrap_or_default().trim().to_string())
}

/// The `target/<profile>/` binaries produced by the bin targets of the given
/// packages. Only `cargo build` (not check/test) declares these outputs so
/// the artifact cache has something real to ship around.
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
                let mut name = target.name.clone();
                if cfg!(windows) {
                    name.push_str(".exe");
                }
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
