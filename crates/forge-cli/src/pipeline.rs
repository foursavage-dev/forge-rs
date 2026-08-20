use forge_executor::Task;
use forge_graph::{BuildGraph, NodeId};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PipelineTaskDef {
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub cache: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PipelineConfig {
    #[serde(flatten)]
    pub tasks: HashMap<String, PipelineTaskDef>,
}

pub struct PipelineResolver;

impl PipelineResolver {
    pub fn build_pipeline_graph(
        config: &PipelineConfig,
        package_tasks: &HashMap<String, Vec<Task>>,
        package_deps: &HashMap<String, Vec<String>>,
    ) -> Result<BuildGraph<Task>, anyhow::Error> {
        let mut graph = BuildGraph::new();
        let mut node_registry: HashMap<(String, String), NodeId> = HashMap::new();

        for (pkg_name, tasks) in package_tasks {
            for task in tasks {
                let node_id = graph.add_node(task.clone());
                node_registry.insert((pkg_name.clone(), task.label.clone()), node_id);
            }
        }

        for ((pkg_name, task_label), &current_node_id) in &node_registry {
            if let Some(task_def) = config.tasks.get(task_label) {
                for dep_pattern in &task_def.depends_on {
                    if let Some(target_task) = dep_pattern.strip_prefix('^') {
                        if let Some(upstream_pkgs) = package_deps.get(pkg_name) {
                            for upstream_pkg in upstream_pkgs {
                                if let Some(&dep_node_id) = node_registry
                                    .get(&(upstream_pkg.clone(), target_task.to_string()))
                                {
                                    let _ = graph.add_dependency(dep_node_id, current_node_id);
                                }
                            }
                        }
                    } else if let Some(&same_pkg_dep_id) =
                        node_registry.get(&(pkg_name.clone(), dep_pattern.clone()))
                    {
                        let _ = graph.add_dependency(same_pkg_dep_id, current_node_id);
                    }
                }
            }
        }

        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_executor::CommandSpec;

    #[test]
    fn test_pipeline_resolver_topological_rules() {
        let mut config = PipelineConfig::default();
        config.tasks.insert(
            "test".to_string(),
            PipelineTaskDef {
                depends_on: vec!["^build".to_string(), "lint".to_string()],
                inputs: vec!["src/**".to_string()],
                outputs: vec![],
                cache: Some(true),
            },
        );

        let mut package_tasks = HashMap::new();
        let spec_build = CommandSpec::new("build");
        let spec_test = CommandSpec::new("test");
        let spec_lint = CommandSpec::new("lint");

        package_tasks.insert(
            "core".to_string(),
            vec![Task::new("build", "core build", spec_build.clone())],
        );
        package_tasks.insert(
            "app".to_string(),
            vec![
                Task::new("build", "app build", spec_build),
                Task::new("lint", "app lint", spec_lint),
                Task::new("test", "app test", spec_test),
            ],
        );

        let mut package_deps = HashMap::new();
        package_deps.insert("app".to_string(), vec!["core".to_string()]);

        let graph =
            PipelineResolver::build_pipeline_graph(&config, &package_tasks, &package_deps).unwrap();

        assert_eq!(graph.len(), 4);
    }
}
