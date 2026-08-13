//! `forge-backend-rust`: turn a Cargo workspace into Forge build tasks.
//!
//! The backend reads the workspace with `cargo metadata`, maps every package
//! into a `forge-graph` node (edges follow `resolve` dependencies), and
//! produces a [`Task`] graph whose commands are `cargo build --package X` /
//! `cargo check --package X` (or your `$CARGO` override) run from the
//! workspace root.
//!
//! When caching is enabled, each task carries a fingerprint combining the
//! package's own inputs (see [`fingerprint`]) with the fingerprints of its
//! direct dependencies, so a change in `core` invalidates `core` *and* every
//! package that depends on it.

use std::collections::HashMap;
use std::path::PathBuf;

use cargo_metadata::PackageId;
use forge_core::project::Project;
use forge_executor::{CacheEntry, CommandSpec, Task};
use forge_graph::{BuildGraph, NodeId};

pub mod fingerprint;

/// The Cargo profile all Forge tasks build with.
pub const PROFILE: &str = "dev";

/// Build-mode errors: metadata/IO problems and toolchain issues.
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    /// A file could not be read while fingerprinting.
    #[error("cannot read `{}`: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// A build tool (cargo/rustc) could not be queried.
    #[error("toolchain query failed: {0}")]
    Toolchain(String),
    /// The package graph was not structurally valid.
    #[error("invalid package graph: {0}")]
    InvalidGraph(#[from] forge_graph::GraphError),
    /// Anything else (e.g. a package listed by metadata but missing).
    #[error("{0}")]
    Message(String),
}

/// What kind of task graph to produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    /// `cargo build` — full compilation, produces artifacts.
    Build,
    /// `cargo check` — type-check only, no artifacts.
    Check,
    /// `cargo test --package X` — compile and run each package's tests.
    Test,
}

impl BuildMode {
    pub fn cargo_subcommand(&self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Check => "check",
            Self::Test => "test",
        }
    }

    pub fn as_str(&self) -> &'static str {
        self.cargo_subcommand()
    }
}

/// The Rust/Cargo backend.
#[derive(Debug, Clone)]
pub struct RustBackend {
    /// Concatenated tool versions, folded into every fingerprint so a
    /// toolchain upgrade invalidates the cache.
    toolchain: String,
}

impl RustBackend {
    /// Detect the toolchain (cargo + rustc versions) at creation time.
    pub fn new() -> Result<Self, BackendError> {
        let cargo = tool_version("cargo", &["--version"])?;
        let rustc = tool_version("rustc", &["--version"])
            .unwrap_or_else(|_| "rustc:unavailable".to_string());
        Ok(Self {
            toolchain: format!("{cargo}|{rustc}"),
        })
    }

    /// Build a task graph from the project's package graph.
    ///
    /// Node order and edge structure mirror `package_graph` exactly (via
    /// `map_nodes`). Task ordering is therefore identical to the package
    /// graph's topological ordering, with `cargo build --package <name>`
    /// as the leaf command and dependencies pre-built.
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

        // 1. Compute per-package fingerprints in topological order so
        //    dependency fingerprints are always available.
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

        // 2. Map the package graph onto a task graph.
        let cargo = std::env::var_os("CARGO")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("cargo"));
        let cargo = cargo.to_string_lossy().into_owned();
        let task_graph = package_graph.map_nodes(|id, package_id| {
            let package = project
                .package(package_id)
                .expect("map_nodes only visits metadata packages");
            let spec = CommandSpec::new(&cargo)
                .arg(mode.cargo_subcommand())
                .arg("--package")
                .arg(package.name.to_string())
                .cwd(&workspace_root);
            let description = spec.command_line();
            let mut task = Task::new(package.name.to_string(), description, spec);
            if caching {
                let fingerprint = fingerprints
                    .get(&id)
                    .expect("all fingerprints are computed before mapping");
                task = task.with_cache(CacheEntry {
                    key: format!("v1/{namespace}/{}/{}", mode.as_str(), package.name),
                    fingerprint: fingerprint.clone(),
                });
            }
            task
        });

        Ok(task_graph)
    }
}

/// Run a tool with arguments and return its first output line, trimmed.
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
