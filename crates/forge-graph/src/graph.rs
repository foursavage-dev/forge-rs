use std::collections::{HashSet, VecDeque};

use crate::error::GraphError;
use crate::state::TaskState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for NodeId {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

#[derive(Debug)]
pub struct Node<T> {
    pub id: NodeId,

    pub payload: T,

    pub state: TaskState,
}

#[derive(Debug)]
pub struct BuildGraph<T> {
    nodes: Vec<Node<T>>,
    deps: Vec<Vec<NodeId>>,
    dependents: Vec<Vec<NodeId>>,
}

impl<T> Default for BuildGraph<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> BuildGraph<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            deps: Vec::new(),
            dependents: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn add_node(&mut self, payload: T) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            id,
            payload,
            state: TaskState::Pending,
        });
        self.deps.push(Vec::new());
        self.dependents.push(Vec::new());
        id
    }

    pub fn nodes(&self) -> &[Node<T>] {
        &self.nodes
    }

    pub fn node(&self, id: NodeId) -> Option<&Node<T>> {
        self.nodes.get(id.0)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.nodes.get_mut(id.0)
    }

    pub fn state(&self, id: NodeId) -> Result<TaskState, GraphError> {
        self.node(id)
            .map(|node| node.state)
            .ok_or(GraphError::MissingNode(id))
    }

    pub fn set_state(&mut self, id: NodeId, state: TaskState) -> Result<(), GraphError> {
        let node = self.node_mut(id).ok_or(GraphError::MissingNode(id))?;
        node.state = state;
        Ok(())
    }

    pub fn add_dependency(
        &mut self,
        dependency: NodeId,
        dependent: NodeId,
    ) -> Result<(), GraphError> {
        if self.node(dependency).is_none() {
            return Err(GraphError::MissingNode(dependency));
        }
        if self.node(dependent).is_none() {
            return Err(GraphError::MissingNode(dependent));
        }
        if dependency == dependent {
            return Err(GraphError::SelfDependency(dependency));
        }
        if self.reaches(dependent, dependency) {
            return Err(GraphError::Cycle {
                dependency,
                dependent,
            });
        }
        self.deps[dependent.0].push(dependency);
        self.dependents[dependency.0].push(dependent);
        Ok(())
    }

    pub fn deps(&self, id: NodeId) -> Result<&[NodeId], GraphError> {
        self.deps
            .get(id.0)
            .map(Vec::as_slice)
            .ok_or(GraphError::MissingNode(id))
    }

    pub fn dependents(&self, id: NodeId) -> Result<&[NodeId], GraphError> {
        self.dependents
            .get(id.0)
            .map(Vec::as_slice)
            .ok_or(GraphError::MissingNode(id))
    }

    pub fn is_ready(&self, id: NodeId) -> Result<bool, GraphError> {
        Ok(self
            .deps(id)?
            .iter()
            .all(|dep| self.state(*dep).is_ok_and(TaskState::is_successful)))
    }

    pub fn ready_nodes(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|node| {
                node.state == TaskState::Pending && self.is_ready(node.id).unwrap_or(false)
            })
            .map(|node| node.id)
            .collect()
    }

    pub fn is_blocked(&self, id: NodeId) -> Result<bool, GraphError> {
        Ok(self
            .deps(id)?
            .iter()
            .any(|dep| self.state(*dep).is_ok_and(TaskState::is_unsuccessful)))
    }

    pub fn mark_failed(&mut self, id: NodeId) -> Result<(), GraphError> {
        self.set_state(id, TaskState::Failed)?;
        let mut queue = VecDeque::from([id]);
        while let Some(current) = queue.pop_front() {
            let dependents = self.dependents(current)?.to_vec();
            for dependent in dependents {
                if self.state(dependent)? != TaskState::Cancelled {
                    self.set_state(dependent, TaskState::Cancelled)?;
                    queue.push_back(dependent);
                }
            }
        }
        Ok(())
    }

    pub fn topological_order(&self) -> Vec<NodeId> {
        let mut indegree: Vec<usize> = self.deps.iter().map(Vec::len).collect();
        let mut ready: VecDeque<NodeId> = self
            .deps
            .iter()
            .enumerate()
            .filter(|(_, deps)| deps.is_empty())
            .map(|(index, _)| NodeId(index))
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(id) = ready.pop_front() {
            order.push(id);
            if let Ok(dependents) = self.dependents(id) {
                for dependent in dependents {
                    indegree[dependent.0] -= 1;
                    if indegree[dependent.0] == 0 {
                        ready.push_back(*dependent);
                    }
                }
            }
        }
        order
    }

    pub fn levels(&self) -> Vec<Vec<NodeId>> {
        let mut level_of: Vec<usize> = vec![0; self.nodes.len()];
        let mut levels: Vec<Vec<NodeId>> = Vec::new();
        for id in self.topological_order() {
            let depth = self
                .deps(id)
                .unwrap_or_default()
                .iter()
                .map(|dep| level_of[dep.0] + 1)
                .max()
                .unwrap_or(0);
            level_of[id.0] = depth;
            while levels.len() <= depth {
                levels.push(Vec::new());
            }
            levels[depth].push(id);
        }
        levels
    }

    pub fn map_nodes<R>(&self, mut f: impl FnMut(NodeId, &T) -> R) -> BuildGraph<R> {
        let mut out = BuildGraph::new();
        for node in &self.nodes {
            out.add_node(f(node.id, &node.payload));
        }
        for index in 0..self.nodes.len() {
            let id = NodeId(index);
            if let Ok(deps) = self.deps(id) {
                for dep in deps {
                    out.add_dependency(*dep, id)
                        .expect("mapped graph inherits the DAG structure");
                }
            }
        }
        out
    }

    pub fn reset_states(&mut self) {
        for node in &mut self.nodes {
            node.state = TaskState::Pending;
        }
    }

    pub fn validate(&self) -> Result<(), GraphError> {
        for (index, (deps, dependents)) in self.deps.iter().zip(&self.dependents).enumerate() {
            let id = NodeId(index);
            for dep in deps {
                if dep.0 >= self.nodes.len() {
                    return Err(GraphError::MissingNode(*dep));
                }
                if dep == &id {
                    return Err(GraphError::SelfDependency(id));
                }
            }
            for dependent in dependents {
                if dependent.0 >= self.nodes.len() {
                    return Err(GraphError::MissingNode(*dependent));
                }
            }
        }
        if self.topological_order().len() != self.nodes.len() {
            return Err(GraphError::Cycle {
                dependency: NodeId(0),
                dependent: NodeId(0),
            });
        }
        Ok(())
    }

    fn reaches(&self, from: NodeId, to: NodeId) -> bool {
        let mut seen: HashSet<NodeId> = HashSet::new();
        let mut queue = VecDeque::from([from]);
        while let Some(id) = queue.pop_front() {
            if id == to {
                return true;
            }
            if !seen.insert(id) {
                continue;
            }
            if let Ok(dependents) = self.dependents(id) {
                queue.extend(dependents.iter().copied());
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string_graph(edges: &[(NodeId, NodeId)]) -> BuildGraph<String> {
        let mut graph = BuildGraph::new();
        for _ in 0..3 {
            graph.add_node(String::new());
        }
        for &(dependency, dependent) in edges {
            graph
                .add_dependency(dependency, dependent)
                .expect("edge must be added");
        }
        graph
    }

    #[test]
    fn add_and_query_nodes() {
        let mut graph = BuildGraph::new();
        let a = graph.add_node("a");
        let b = graph.add_node("b");

        assert_eq!(graph.len(), 2);
        assert!(!graph.is_empty());
        assert_eq!(graph.node(a).map(|n| n.payload), Some("a"));
        assert_eq!(graph.state(b), Ok(TaskState::Pending));
        assert!(graph.node(NodeId(99)).is_none());
        assert_eq!(
            graph.state(NodeId(99)),
            Err(GraphError::MissingNode(NodeId(99)))
        );
    }

    #[test]
    fn edges_are_tracked_in_both_directions() {
        let graph = string_graph(&[(NodeId(0), NodeId(1)), (NodeId(1), NodeId(2))]);
        assert_eq!(graph.deps(NodeId(2)), Ok(&[NodeId(1)][..]));
        assert_eq!(graph.deps(NodeId(0)), Ok(&[][..]));
        assert_eq!(graph.dependents(NodeId(0)), Ok(&[NodeId(1)][..]));
        assert_eq!(graph.dependents(NodeId(2)), Ok(&[][..]));
    }

    #[test]
    fn rejects_missing_self_and_cyclic_edges() {
        let mut graph = string_graph(&[(NodeId(0), NodeId(1)), (NodeId(1), NodeId(2))]);

        assert_eq!(
            graph.add_dependency(NodeId(7), NodeId(0)),
            Err(GraphError::MissingNode(NodeId(7)))
        );
        assert_eq!(
            graph.add_dependency(NodeId(1), NodeId(1)),
            Err(GraphError::SelfDependency(NodeId(1)))
        );
        assert_eq!(
            graph.add_dependency(NodeId(2), NodeId(0)),
            Err(GraphError::Cycle {
                dependency: NodeId(2),
                dependent: NodeId(0)
            })
        );

        graph
            .add_dependency(NodeId(0), NodeId(2))
            .expect("parallel edges are allowed");

        assert_eq!(
            graph.add_dependency(NodeId(2), NodeId(0)),
            Err(GraphError::Cycle {
                dependency: NodeId(2),
                dependent: NodeId(0)
            })
        );
    }

    #[test]
    fn topological_order_respects_dependencies() {
        let mut graph = BuildGraph::new();
        for _ in 0..4 {
            graph.add_node(String::new());
        }
        for &(dependency, dependent) in &[
            (NodeId(0), NodeId(2)),
            (NodeId(1), NodeId(2)),
            (NodeId(2), NodeId(3)),
        ] {
            graph
                .add_dependency(dependency, dependent)
                .expect("edge must be added");
        }

        let extra = graph.add_node("extra".to_string());

        let order = graph.topological_order();
        let position = |id: NodeId| order.iter().position(|&n| n == id).unwrap();
        assert!(position(NodeId(0)) < position(NodeId(2)));
        assert!(position(NodeId(1)) < position(NodeId(2)));
        assert!(position(NodeId(2)) < position(NodeId(3)));
        assert_eq!(order.len(), 5);
        assert!(order.contains(&extra));
    }

    #[test]
    fn levels_place_diamonds_correctly() {
        let mut graph = BuildGraph::new();
        for _ in 0..5 {
            graph.add_node(());
        }

        for (dependency, dependent) in [
            (NodeId(0), NodeId(2)),
            (NodeId(1), NodeId(2)),
            (NodeId(2), NodeId(3)),
        ] {
            graph
                .add_dependency(dependency, dependent)
                .expect("edge must be added");
        }

        let levels = graph.levels();
        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0], vec![NodeId(0), NodeId(1), NodeId(4)]);
        assert_eq!(levels[1], vec![NodeId(2)]);
        assert_eq!(levels[2], vec![NodeId(3)]);
    }

    #[test]
    fn readiness_requires_successful_dependencies() {
        let mut graph = string_graph(&[(NodeId(0), NodeId(1)), (NodeId(1), NodeId(2))]);

        assert!(!graph.is_ready(NodeId(2)).unwrap());
        assert_eq!(graph.ready_nodes(), vec![NodeId(0)]);

        graph.set_state(NodeId(0), TaskState::Succeeded).unwrap();
        assert!(graph.is_ready(NodeId(1)).unwrap());
        assert!(!graph.is_ready(NodeId(2)).unwrap());
        assert_eq!(graph.ready_nodes(), vec![NodeId(1)]);

        graph.set_state(NodeId(0), TaskState::Failed).unwrap();
        assert!(!graph.is_ready(NodeId(1)).unwrap());
        assert_eq!(graph.ready_nodes(), vec![]);
    }

    #[test]
    fn cached_and_skipped_dependencies_unblock_dependents() {
        let mut graph = string_graph(&[(NodeId(0), NodeId(1))]);
        for state in [TaskState::Cached, TaskState::Skipped] {
            assert!(
                !graph.is_ready(NodeId(1)).unwrap(),
                "pending dependency must not make a node ready"
            );
            graph.set_state(NodeId(0), state).unwrap();
            assert!(graph.is_ready(NodeId(1)).unwrap());
            graph.set_state(NodeId(0), TaskState::Pending).unwrap();
        }
    }

    #[test]
    fn failure_propagates_to_transitive_dependents() {
        let mut graph = string_graph(&[(NodeId(0), NodeId(1)), (NodeId(1), NodeId(2))]);
        graph.set_state(NodeId(1), TaskState::Running).unwrap();

        graph.mark_failed(NodeId(0)).unwrap();

        assert_eq!(graph.state(NodeId(0)), Ok(TaskState::Failed));
        assert_eq!(graph.state(NodeId(1)), Ok(TaskState::Cancelled));
        assert_eq!(graph.state(NodeId(2)), Ok(TaskState::Cancelled));
        assert!(graph.is_blocked(NodeId(1)).unwrap());
        assert!(!graph.is_ready(NodeId(1)).unwrap());
    }

    #[test]
    fn failure_does_not_cancel_nodes_before_the_failure_point() {
        let mut graph = BuildGraph::new();
        for _ in 0..4 {
            graph.add_node(String::new());
        }
        for &(dependency, dependent) in &[
            (NodeId(0), NodeId(1)),
            (NodeId(1), NodeId(2)),
            (NodeId(2), NodeId(3)),
        ] {
            graph
                .add_dependency(dependency, dependent)
                .expect("edge must be added");
        }
        graph.set_state(NodeId(0), TaskState::Succeeded).unwrap();
        graph.set_state(NodeId(1), TaskState::Running).unwrap();

        graph.mark_failed(NodeId(2)).unwrap();

        assert_eq!(graph.state(NodeId(0)), Ok(TaskState::Succeeded));
        assert_eq!(graph.state(NodeId(1)), Ok(TaskState::Running));
        assert_eq!(graph.state(NodeId(2)), Ok(TaskState::Failed));
        assert_eq!(graph.state(NodeId(3)), Ok(TaskState::Cancelled));
    }

    #[test]
    fn validate_accepts_valid_graphs() {
        let graph = string_graph(&[(NodeId(0), NodeId(1)), (NodeId(1), NodeId(2))]);
        assert_eq!(graph.validate(), Ok(()));
    }
}
