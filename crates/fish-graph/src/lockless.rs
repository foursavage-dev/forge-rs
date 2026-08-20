use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Default)]
pub struct LocklessGraphNode {
    pub id: String,
    pub dependencies: Vec<String>,
    pub reverse_dependencies: Vec<String>,
    pub execution_weight: u64,
}

#[derive(Debug, Clone, Default)]
pub struct LocklessDependencyGraph {
    nodes: HashMap<String, LocklessGraphNode>,
}

impl LocklessDependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn insert_node(&mut self, id: &str, dependencies: &[String], weight: u64) {
        let node = self.nodes.entry(id.to_string()).or_default();
        node.id = id.to_string();
        node.dependencies = dependencies.to_vec();
        node.execution_weight = weight;

        for dep in dependencies {
            let dep_node = self.nodes.entry(dep.clone()).or_default();
            dep_node.id = dep.clone();
            dep_node.reverse_dependencies.push(id.to_string());
        }
    }

    pub fn get_node(&self, id: &str) -> Option<&LocklessGraphNode> {
        self.nodes.get(id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn transitive_dependencies(&self, id: &str) -> Vec<String> {
        let mut visited = BTreeSet::new();
        let mut stack = vec![id.to_string()];

        while let Some(current) = stack.pop() {
            if let Some(node) = self.nodes.get(&current) {
                for dep in &node.dependencies {
                    if visited.insert(dep.clone()) {
                        stack.push(dep.clone());
                    }
                }
            }
        }

        visited.into_iter().collect()
    }

    pub fn compute_critical_path(&self) -> Vec<String> {
        let mut longest_paths: HashMap<String, (u64, Vec<String>)> = HashMap::new();

        for (id, node) in &self.nodes {
            if node.dependencies.is_empty() {
                longest_paths.insert(id.clone(), (node.execution_weight, vec![id.clone()]));
            }
        }

        let mut sorted_nodes: Vec<_> = self.nodes.keys().cloned().collect();
        sorted_nodes.sort();

        for id in sorted_nodes {
            if let Some(node) = self.nodes.get(&id) {
                let mut max_dep_weight = 0;
                let mut best_prefix = Vec::new();

                for dep in &node.dependencies {
                    if let Some((w, path)) = longest_paths.get(dep)
                        && *w > max_dep_weight
                    {
                        max_dep_weight = *w;
                        best_prefix = path.clone();
                    }
                }

                best_prefix.push(id.clone());
                longest_paths.insert(
                    id.clone(),
                    (max_dep_weight + node.execution_weight, best_prefix),
                );
            }
        }

        longest_paths
            .into_values()
            .max_by_key(|(weight, _)| *weight)
            .map(|(_, path)| path)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockless_graph_insertion_and_transitive_deps() {
        let mut graph = LocklessDependencyGraph::new();
        graph.insert_node("core", &[], 10);
        graph.insert_node("utils", &["core".to_string()], 20);
        graph.insert_node("cli", &["utils".to_string()], 30);

        assert_eq!(graph.node_count(), 3);
        let deps = graph.transitive_dependencies("cli");
        assert!(deps.contains(&"core".to_string()));
        assert!(deps.contains(&"utils".to_string()));

        let crit_path = graph.compute_critical_path();
        assert_eq!(crit_path, vec!["core", "utils", "cli"]);
    }
}
