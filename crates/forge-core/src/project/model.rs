use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use cargo_metadata::camino::Utf8Path;
use cargo_metadata::{DependencyKind, Metadata, MetadataCommand, Package, PackageId};
use forge_graph::{BuildGraph, NodeId};

use crate::error::{ForgeError, Result};
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
