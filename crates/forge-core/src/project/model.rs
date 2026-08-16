use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use cargo_metadata::camino::Utf8Path;
use cargo_metadata::{DependencyKind, Metadata, MetadataCommand, Package, PackageId};
use forge_graph::{BuildGraph, NodeId};

use crate::error::{ErrorContext, ErrorKind, ErrorSeverity, ForgeError, Result};
use crate::project::detect::find_manifest;

#[derive(Debug, Clone)]
pub struct Project {
    manifest_path: PathBuf,
    metadata: Metadata,
}

impl Project {
    pub fn discover(start: &Path) -> Result<Option<Project>> {
        let Some(manifest) = find_manifest(start) else {
            return Ok(None);
        };
        Self::load(&manifest).map(Some)
    }

    pub fn load(manifest_path: &Path) -> Result<Project> {
        let manifest = Utf8Path::from_path(manifest_path).ok_or_else(|| {
            ForgeError::new(
                ErrorKind::NonUtf8ManifestPath(manifest_path.display().to_string()),
                ErrorContext::new("load_project", "project_loader")
                    .with_file(manifest_path.display().to_string())
                    .with_severity(ErrorSeverity::Error),
            )
        })?;

        let metadata = MetadataCommand::new()
            .manifest_path(manifest)
            .exec()
            .map_err(|source| {
                ForgeError::new(
                    ErrorKind::CargoMetadata(source.to_string()),
                    ErrorContext::new("load_metadata", "project_loader")
                        .with_file(manifest_path.display().to_string())
                        .with_severity(ErrorSeverity::Error),
                )
            })?;

        Ok(Project {
            manifest_path: manifest_path.to_path_buf(),
            metadata,
        })
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn workspace_root(&self) -> &Utf8Path {
        &self.metadata.workspace_root
    }

    pub fn workspace_members(&self) -> &[PackageId] {
        &self.metadata.workspace_members
    }

    pub fn default_workspace_members(&self) -> &[PackageId] {
        &self.metadata.workspace_default_members
    }

    pub fn root_package(&self) -> Option<&Package> {
        self.metadata.root_package()
    }

    pub fn is_workspace(&self) -> bool {
        self.metadata.workspace_members.len() > 1
    }

    pub fn package(&self, id: &PackageId) -> Option<&Package> {
        self.metadata
            .packages
            .iter()
            .find(|package| &package.id == id)
    }

    /// Maps a set of changed file paths to the workspace packages that own
    /// them. A file belongs to a package when it lives under the package's
    /// manifest directory or is one of the package's declared targets.
    /// Returns `None` when the caller should treat the whole workspace as
    /// affected (e.g. a workspace-root `Cargo.toml`/`Cargo.lock` change).
    pub fn packages_for_paths(&self, paths: &[&Path]) -> Option<Vec<PackageId>> {
        let members: Vec<&Package> = self
            .metadata
            .workspace_members
            .iter()
            .filter_map(|id| self.package(id))
            .collect();

        let mut owners: Vec<&Package> = Vec::new();
        for path in paths {
            let path = plain_path(path);
            let mut best: Option<(&Package, usize)> = None;
            for package in &members {
                let package_dir = package
                    .manifest_path
                    .parent()
                    .map(|parent| plain_path(&PathBuf::from(parent.as_str())))
                    .unwrap_or_else(|| PathBuf::from("."));
                let owned = path.starts_with(&package_dir)
                    || package
                        .targets
                        .iter()
                        .any(|target| plain_path(Path::new(target.src_path.as_str())) == path);
                if owned {
                    let specificity = package_dir.components().count();
                    if best.map(|(_, len)| specificity > len).unwrap_or(true) {
                        best = Some((package, specificity));
                    }
                }
            }
            let (package, _) = best?;
            if !owners.iter().any(|p| p.id == package.id) {
                owners.push(package);
            }
        }

        let mut affected: Vec<PackageId> = Vec::new();
        for id in &self.metadata.workspace_members {
            if owners.iter().any(|p| &p.id == id) {
                affected.push(id.clone());
            }
        }
        Some(affected)
    }

    pub fn build_order(&self) -> Vec<PackageId> {
        let Some(resolve) = &self.metadata.resolve else {
            return self.metadata.workspace_members.clone();
        };

        let members: HashSet<&PackageId> = self.metadata.workspace_members.iter().collect();
        let mut depended_on_by: HashMap<PackageId, Vec<PackageId>> = HashMap::new();
        let mut indegree: HashMap<PackageId, usize> = HashMap::new();

        for id in &self.metadata.workspace_members {
            indegree.insert(id.clone(), 0);
            depended_on_by.insert(id.clone(), Vec::new());
        }

        for node in &resolve.nodes {
            if !members.contains(&node.id) {
                continue;
            }
            for dep in &node.deps {
                if members.contains(&dep.pkg) && node.id != dep.pkg {
                    depended_on_by
                        .get_mut(&dep.pkg)
                        .expect("workspace members are pre-registered")
                        .push(node.id.clone());
                    *indegree
                        .get_mut(&node.id)
                        .expect("workspace members are pre-registered") += 1;
                }
            }
        }

        let mut name_to_id: HashMap<String, PackageId> = HashMap::new();
        for id in &self.metadata.workspace_members {
            let Some(package) = self.package(id) else {
                continue;
            };
            name_to_id.insert(package.name.to_string(), id.clone());
        }

        let mut ready: BinaryHeap<Reverse<String>> = BinaryHeap::new();
        for id in &self.metadata.workspace_members {
            if indegree[id] == 0 {
                ready.push(Reverse(
                    self.package(id)
                        .expect("members are present in metadata")
                        .name
                        .to_string(),
                ));
            }
        }

        let mut order: Vec<PackageId> = Vec::with_capacity(members.len());
        while let Some(Reverse(name)) = ready.pop() {
            let id = name_to_id
                .get(&name)
                .expect("ready names come from the name_to_id map")
                .clone();
            order.push(id.clone());
            let Some(dependents) = depended_on_by.get(&id) else {
                continue;
            };
            for dependent in dependents {
                let degree = indegree
                    .get_mut(dependent)
                    .expect("workspace members are pre-registered");
                *degree -= 1;
                if *degree == 0 {
                    ready.push(Reverse(
                        self.package(dependent)
                            .expect("members are present in metadata")
                            .name
                            .to_string(),
                    ));
                }
            }
        }

        if order.len() < members.len() {
            let visited: HashSet<PackageId> = order.iter().cloned().collect();
            for id in &self.metadata.workspace_members {
                if !visited.contains(id) {
                    order.push(id.clone());
                }
            }
        }

        order
    }

    pub fn build_graph(&self) -> Result<BuildGraph<PackageId>> {
        self.build_graph_with(false)
    }

    pub fn build_test_graph(&self) -> Result<BuildGraph<PackageId>> {
        self.build_graph_with(true)
    }

    fn build_graph_with(&self, include_dev: bool) -> Result<BuildGraph<PackageId>> {
        let mut graph = BuildGraph::new();
        let mut ids: HashMap<&PackageId, NodeId> = HashMap::new();
        for id in &self.metadata.workspace_members {
            ids.insert(id, graph.add_node(id.clone()));
        }

        let Some(resolve) = &self.metadata.resolve else {
            return Ok(graph);
        };
        for node in &resolve.nodes {
            let Some(&dependent) = ids.get(&node.id) else {
                continue;
            };
            for dep in &node.deps {
                let Some(&dependency) = ids.get(&dep.pkg) else {
                    continue;
                };
                if dependency == dependent {
                    continue;
                }
                let is_dev_only = dep
                    .dep_kinds
                    .iter()
                    .all(|info| info.kind == DependencyKind::Development);
                if is_dev_only && !include_dev {
                    continue;
                }
                if graph.add_dependency(dependency, dependent).is_err() {
                    debug_assert!(is_dev_only);
                }
            }
        }
        Ok(graph)
    }
}

/// Strips the `\\?\` extended-length prefix that `std::fs::canonicalize`
/// (and therefore cargo metadata) may attach on Windows, so that paths from
/// different sources compare identically.
fn plain_path(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    if cfg!(windows) {
        if let Some(stripped) = text.strip_prefix(r"\\?\") {
            return PathBuf::from(stripped);
        }
    }
    path.to_path_buf()
}
