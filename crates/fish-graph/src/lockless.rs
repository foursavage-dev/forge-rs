use std::collections::{BTreeSet, HashMap, HashSet};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LocklessError {
    /// A dependency cycle was detected while computing the critical path.
    #[error("dependency cycle detected at node `{0}`")]
    Cycle(String),
}

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
        // Drop stale reverse edges left behind by a previous insertion of
        // `id` with a different dependency set.
        let previous_deps: Option<Vec<String>> =
            self.nodes.get(id).map(|node| node.dependencies.clone());
        if let Some(previous_deps) = previous_deps {
            for old_dep in &previous_deps {
                if let Some(dep_node) = self.nodes.get_mut(old_dep) {
                    dep_node.reverse_dependencies.retain(|rd| rd.as_str() != id);
                }
            }
        }

        let node = self.nodes.entry(id.to_string()).or_default();
        node.id = id.to_string();
        node.dependencies = dependencies.to_vec();
        node.execution_weight = weight;

        for dep in dependencies {
            let dep_node = self.nodes.entry(dep.clone()).or_default();
            dep_node.id = dep.clone();
            if !dep_node.reverse_dependencies.contains(&id.to_string()) {
                dep_node.reverse_dependencies.push(id.to_string());
            }
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

    pub fn compute_critical_path(&self) -> Result<Vec<String>, LocklessError> {
        let mut memo: HashMap<String, (u64, Vec<String>)> = HashMap::new();
        let mut visiting: HashSet<String> = HashSet::new();

        fn longest_path(
            id: &str,
            nodes: &HashMap<String, LocklessGraphNode>,
            memo: &mut HashMap<String, (u64, Vec<String>)>,
            visiting: &mut HashSet<String>,
        ) -> Result<(u64, Vec<String>), LocklessError> {
            if let Some(cached) = memo.get(id) {
                return Ok(cached.clone());
            }
            if !visiting.insert(id.to_string()) {
                return Err(LocklessError::Cycle(id.to_string()));
            }

            let mut max_dep_weight = 0;
            let mut best_prefix = Vec::new();

            if let Some(node) = nodes.get(id) {
                for dep in &node.dependencies {
                    let (w, path) = longest_path(dep, nodes, memo, visiting)?;
                    if w > max_dep_weight {
                        max_dep_weight = w;
                        best_prefix = path;
                    }
                }
                best_prefix.push(id.to_string());
                let total_weight = max_dep_weight + node.execution_weight;
                let result = (total_weight, best_prefix);
                memo.insert(id.to_string(), result.clone());
                visiting.remove(id);
                Ok(result)
            } else {
                visiting.remove(id);
                Ok((0, vec![id.to_string()]))
            }
        }

        let mut best_path = Vec::new();
        let mut max_weight = 0;

        // Iterate in sorted order so the result is deterministic even when
        // several paths share the same total weight.
        let mut ids: Vec<&String> = self.nodes.keys().collect();
        ids.sort();

        for id in ids {
            let (w, path) = longest_path(id, &self.nodes, &mut memo, &mut visiting)?;
            if w > max_weight {
                max_weight = w;
                best_path = path;
            }
        }

        Ok(best_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deps(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| id.to_string()).collect()
    }

    #[test]
    fn test_lockless_graph_insertion_and_transitive_deps() {
        let mut graph = LocklessDependencyGraph::new();
        graph.insert_node("core", &[], 10);
        graph.insert_node("utils", &deps(&["core"]), 20);
        graph.insert_node("cli", &deps(&["utils"]), 30);

        assert_eq!(graph.node_count(), 3);
        let transitive = graph.transitive_dependencies("cli");
        assert!(transitive.contains(&"core".to_string()));
        assert!(transitive.contains(&"utils".to_string()));

        let crit_path = graph.compute_critical_path().unwrap();
        assert_eq!(crit_path, vec!["core", "utils", "cli"]);
    }

    #[test]
    fn compute_critical_path_picks_heaviest_chain() {
        let mut graph = LocklessDependencyGraph::new();
        graph.insert_node("a", &[], 1);
        graph.insert_node("b", &[], 1);
        graph.insert_node("c", &deps(&["a"]), 1);
        graph.insert_node("d", &deps(&["b"]), 100);
        graph.insert_node("e", &deps(&["c", "d"]), 1);

        let path = graph.compute_critical_path().unwrap();
        // The chain b -> d -> e (weight 102) dominates a -> c -> e (weight 3).
        assert_eq!(path, vec!["b", "d", "e"]);
    }

    #[test]
    fn compute_critical_path_is_deterministic_across_equivalent_graphs() {
        let build = || {
            let mut graph = LocklessDependencyGraph::new();
            graph.insert_node("root", &deps(&["left", "right"]), 0);
            graph.insert_node("left", &[], 1);
            graph.insert_node("right", &[], 1);
            graph
        };

        let expected = build().compute_critical_path().unwrap();
        for _ in 0..20 {
            assert_eq!(build().compute_critical_path().unwrap(), expected);
        }
    }

    #[test]
    fn compute_critical_path_detects_cycles_instead_of_overflowing() {
        let mut graph = LocklessDependencyGraph::new();
        graph.insert_node("a", &deps(&["b"]), 1);
        graph.insert_node("b", &deps(&["a"]), 1);

        assert!(matches!(
            graph.compute_critical_path(),
            Err(LocklessError::Cycle(_))
        ));

        let mut self_loop = LocklessDependencyGraph::new();
        self_loop.insert_node("x", &deps(&["x"]), 1);
        assert!(matches!(
            self_loop.compute_critical_path(),
            Err(LocklessError::Cycle(_))
        ));
    }

    #[test]
    fn reinserting_node_updates_reverse_dependencies() {
        let mut graph = LocklessDependencyGraph::new();
        graph.insert_node("a", &deps(&["b"]), 1);

        assert_eq!(
            graph.get_node("b").unwrap().reverse_dependencies,
            deps(&["a"])
        );

        // Re-insert `a` with a different dependency set.
        graph.insert_node("a", &deps(&["c"]), 1);

        assert!(graph.get_node("b").unwrap().reverse_dependencies.is_empty());
        assert_eq!(
            graph.get_node("c").unwrap().reverse_dependencies,
            deps(&["a"])
        );
    }

    #[test]
    fn duplicate_insertions_do_not_duplicate_reverse_edges() {
        let mut graph = LocklessDependencyGraph::new();
        graph.insert_node("a", &deps(&["b"]), 1);
        graph.insert_node("a", &deps(&["b"]), 1);

        assert_eq!(
            graph.get_node("b").unwrap().reverse_dependencies,
            deps(&["a"])
        );
    }

    #[test]
    fn empty_graph_has_empty_critical_path() {
        let graph = LocklessDependencyGraph::new();
        assert_eq!(graph.compute_critical_path().unwrap(), Vec::<String>::new());
    }
}
