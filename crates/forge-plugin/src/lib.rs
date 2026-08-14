#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

use forge_core::BuildBackend;
use forge_executor::{CacheEntry, CommandSpec, Task};
use forge_graph::BuildGraph;

pub mod manifest;
pub mod rule;

pub use rule::{PluginRulesManifest, RuleSpec};

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("manifest error: {0}")]
    Manifest(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("graph error: {0}")]
    Graph(#[from] forge_graph::GraphError),
}

#[derive(Debug, Clone, Default)]
pub struct PluginBackend;

impl BuildBackend for PluginBackend {
    fn name(&self) -> &'static str {
        "plugin"
    }
}

impl PluginBackend {
    pub fn new() -> Self {
        Self
    }

    fn compute_rule_fingerprint(root: &Path, rule: &RuleSpec) -> Result<String, std::io::Error> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(rule.name.as_bytes());
        hasher.update(rule.command.as_bytes());
        for arg in &rule.args {
            hasher.update(arg.as_bytes());
        }

        let mut sorted_keys: Vec<_> = rule.env.keys().collect();
        sorted_keys.sort();
        for k in sorted_keys {
            hasher.update(k.as_bytes());
            hasher.update(rule.env[k].as_bytes());
        }

        if rule.inputs.is_empty() {
            hasher.update(b"all_sources");
        } else {
            for pattern in &rule.inputs {
                let target = root.join(pattern);
                if target.is_file() {
                    let bytes = fs::read(&target)?;
                    hasher.update(&bytes);
                } else if target.is_dir() {
                    hash_dir_simple(&target, &mut hasher)?;
                }
            }
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    pub fn build_task_graph(
        &self,
        manifest: &PluginRulesManifest,
        root: &Path,
    ) -> Result<BuildGraph<Task>, PluginError> {
        let mut graph = BuildGraph::new();

        let mut hasher = blake3::Hasher::new();
        hasher.update(root.to_string_lossy().as_bytes());
        let namespace = hasher.finalize().to_hex().to_string()[..12].to_string();

        let mut node_map: HashMap<String, forge_graph::NodeId> = HashMap::new();
        let mut dep_fps: HashMap<String, String> = HashMap::new();

        while node_map.len() < manifest.rules.len() {
            let ready: Vec<&RuleSpec> = manifest
                .rules
                .iter()
                .filter(|r| !node_map.contains_key(&r.name))
                .filter(|r| r.depends_on.iter().all(|dep| node_map.contains_key(dep)))
                .collect();

            if ready.is_empty() {
                let unresolved: Vec<&str> = manifest
                    .rules
                    .iter()
                    .filter(|r| !node_map.contains_key(&r.name))
                    .map(|r| r.name.as_str())
                    .collect();
                return Err(PluginError::Manifest(format!(
                    "dependency cycle or unknown rule dependencies: {}",
                    unresolved.join(", ")
                )));
            }

            for rule in ready {
                let base_fp = Self::compute_rule_fingerprint(root, rule)?;
                let mut member_fps = Vec::new();
                for dep in &rule.depends_on {
                    member_fps.push(
                        dep_fps
                            .get(dep)
                            .ok_or_else(|| {
                                PluginError::Manifest(format!(
                                    "rule `{}` depends on unknown rule `{}`",
                                    rule.name, dep
                                ))
                            })?
                            .clone(),
                    );
                }

                let mut combined_hasher = blake3::Hasher::new();
                combined_hasher.update(base_fp.as_bytes());
                member_fps.sort();
                for fp in member_fps {
                    combined_hasher.update(fp.as_bytes());
                }
                let rule_fp = combined_hasher.finalize().to_hex().to_string();

                let mut spec = CommandSpec::new(&rule.command);
                for arg in &rule.args {
                    spec = spec.arg(arg);
                }
                for (k, v) in &rule.env {
                    spec = spec.env(k, v);
                }
                spec = spec.cwd(root);

                let label = if manifest.name.is_empty() {
                    rule.name.clone()
                } else {
                    format!("{}:{}", manifest.name, rule.name)
                };

                let task = Task::new(label, spec.command_line(), spec).with_cache(CacheEntry {
                    key: format!("plugin/{}/{}", namespace, rule.name),
                    fingerprint: rule_fp.clone(),
                });

                let node_id = graph.add_node(task);
                node_map.insert(rule.name.clone(), node_id);
                dep_fps.insert(rule.name.clone(), rule_fp);
            }
        }

        for rule in &manifest.rules {
            let node_id = node_map[&rule.name];
            for dep in &rule.depends_on {
                graph.add_dependency(node_map[dep], node_id)?;
            }
        }

        Ok(graph)
    }
}

fn hash_dir_simple(dir: &Path, hasher: &mut blake3::Hasher) -> Result<(), std::io::Error> {
    let mut entries = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            hash_dir_simple(&path, hasher)?;
        } else if path.is_file() {
            hasher.update(entry.file_name().to_string_lossy().as_bytes());
            let bytes = fs::read(&path)?;
            hasher.update(&bytes);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_plugin_task_graph_generation() {
        let dir = tempdir().unwrap();
        let forgefile = r#"{
            "name": "custom-engine",
            "rules": [
                {
                    "name": "codegen",
                    "command": "node",
                    "args": ["-e", "process.exit(0)"],
                    "depends_on": []
                },
                {
                    "name": "compile",
                    "command": "node",
                    "args": ["-e", "process.exit(0)"],
                    "depends_on": ["codegen"]
                }
            ]
        }"#;
        fs::write(dir.path().join("Forgefile.json"), forgefile).unwrap();

        let manifest = PluginRulesManifest::discover_or_load(dir.path()).unwrap();
        let backend = PluginBackend::new();
        let graph = backend.build_task_graph(&manifest, dir.path()).unwrap();

        assert_eq!(graph.len(), 2);
        let topo = graph.topological_order();
        assert_eq!(topo.len(), 2);
    }
}
