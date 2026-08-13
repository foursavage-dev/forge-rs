//! Project model built from Cargo metadata.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use cargo_metadata::camino::Utf8Path;
use cargo_metadata::{DependencyKind, Metadata, MetadataCommand, Package, PackageId};
use forge_graph::{BuildGraph, NodeId};

use crate::error::{ForgeError, Result};
use crate::project::detect::find_manifest;

/// A Cargo project (single package or workspace) discovered on disk.
///
/// Forge deliberately exposes Cargo's own metadata types here: this is the
/// interop layer with Cargo, and re-inventing package models would duplicate
/// a stable, well-documented data format.
#[derive(Debug, Clone)]
pub struct Project {
    manifest_path: PathBuf,
    metadata: Metadata,
}

impl Project {
    /// Locate the Cargo project containing `start` by walking up the
    /// directory tree, and load its metadata.
    ///
    /// Returns `Ok(None)` when no `Cargo.toml` exists at or above `start`.
    pub fn discover(start: &Path) -> Result<Option<Project>> {
        let Some(manifest) = find_manifest(start) else {
            return Ok(None);
        };
        Self::load(&manifest).map(Some)
    }

    /// Load the project defined by an explicit `Cargo.toml` manifest path.
    ///
    /// The file does not have to exist; Cargo reports the error if not.
    pub fn load(manifest_path: &Path) -> Result<Project> {
        let manifest = Utf8Path::from_path(manifest_path)
            .ok_or_else(|| ForgeError::NonUtf8ManifestPath(manifest_path.to_path_buf()))?;

        let metadata = MetadataCommand::new()
            .manifest_path(manifest)
            .exec()
            .map_err(|source| ForgeError::CargoMetadata {
                manifest: manifest_path.to_path_buf(),
                source,
            })?;

        Ok(Project {
            manifest_path: manifest_path.to_path_buf(),
            metadata,
        })
    }

    /// Path of the manifest this project was loaded from.
    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    /// Raw Cargo metadata for this project.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Root directory of the workspace this project belongs to.
    pub fn workspace_root(&self) -> &Utf8Path {
        &self.metadata.workspace_root
    }

    /// IDs of all packages in the workspace.
    pub fn workspace_members(&self) -> &[PackageId] {
        &self.metadata.workspace_members
    }

    /// IDs of the workspace packages selected by default (`default-members`).
    pub fn default_workspace_members(&self) -> &[PackageId] {
        &self.metadata.workspace_default_members
    }

    /// The package containing the loaded manifest, if it is a package and not
    /// a virtual workspace manifest.
    pub fn root_package(&self) -> Option<&Package> {
        self.metadata.root_package()
    }

    /// Whether the manifest defines a workspace with more than one member.
    pub fn is_workspace(&self) -> bool {
        self.metadata.workspace_members.len() > 1
    }

    /// Look up a package by its ID.
    pub fn package(&self, id: &PackageId) -> Option<&Package> {
        self.metadata
            .packages
            .iter()
            .find(|package| &package.id == id)
    }

    /// Workspace members ordered so that every member appears after any
    /// workspace member it depends on (topological order, derived from
    /// Cargo's resolved dependency graph).
    ///
    /// Dependency cycles (possible through e.g. dev-dependencies) are broken
    /// deterministically: unvisitable members are appended in manifest order.
    /// This function never loops.
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

        // Edges point from dependency to dependent: a member is ready once all
        // the members it depends on have been placed.
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

        // Deterministic tie-breaking: package names are unique within a
        // workspace, so pop from a min-heap keyed by name.
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

        // Append members that could not be topologically placed.
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

    /// Construct the workspace build graph from Cargo's resolved dependency
    /// graph.
    ///
    /// Only normal and build dependencies contribute edges: Cargo allows
    /// dependency cycles through dev-dependencies (which only matter for
    /// test builds), and those edges are not part of the build order.
    pub fn build_graph(&self) -> Result<BuildGraph<PackageId>> {
        self.build_graph_with(false)
    }

    /// Test-build graph: like [`Self::build_graph`] but also includes
    /// dev-dependency edges, so `forge test` builds nothing its tests need
    /// before running them.
    ///
    /// Dev-dependency edges that would create a cycle are skipped: Cargo
    /// permits dev-dep cycles and resolves them itself, so a missing edge
    /// only costs a slightly less optimal schedule.
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
                    // Only dev edges can trigger this (build edges come from
                    // Cargo's acyclic resolution): leave the edge out rather
                    // than reject the workspace.
                    debug_assert!(is_dev_only);
                }
            }
        }
        Ok(graph)
    }
}
