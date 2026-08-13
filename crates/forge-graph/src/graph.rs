//! The Forge build graph data structure.
//!
//! A [`BuildGraph`] is a directed acyclic graph where an edge
//! `dependency -> dependent` means "`dependency` must finish before
//! `dependent` starts". Nodes carry an arbitrary payload (typically a task
//! description from a language backend) and a [`TaskState`].

use std::collections::{HashSet, VecDeque};

use crate::error::GraphError;
use crate::state::TaskState;

/// Stable identifier of a node inside a [`BuildGraph`].
///
/// Node IDs are compact indices; they are only meaningful within the graph
/// that created them.
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

/// A node in the build graph: one unit of work.
#[derive(Debug)]
pub struct Node<T> {
    /// The node's identifier within its graph.
    pub id: NodeId,
    /// Backend-defined description of the work to perform.
    pub payload: T,
    /// Current lifecycle state.
    pub state: TaskState,
}

/// A directed acyclic graph of build tasks.
///
/// Nodes are stored contiguously and addressed by [`NodeId`]; dependency
/// adjacency is kept for edges in both directions (`deps` and `dependents`),
/// so traversal in either direction is a single slice lookup.
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
    /// Create an empty build graph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            deps: Vec::new(),
            dependents: Vec::new(),
        }
    }

    /// Whether the graph contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Number of nodes in the graph.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Add a node with an initial [`TaskState::Pending`] state.
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

    /// All nodes in the graph, in insertion order.
    pub fn nodes(&self) -> &[Node<T>] {
        &self.nodes
    }

    /// Look up a node by ID.
    pub fn node(&self, id: NodeId) -> Option<&Node<T>> {
        self.nodes.get(id.0)
    }

    /// Mutably look up a node by ID.
    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.nodes.get_mut(id.0)
    }

    /// Current state of the node, or an error if the ID does not exist.
    pub fn state(&self, id: NodeId) -> Result<TaskState, GraphError> {
        self.node(id)
            .map(|node| node.state)
            .ok_or(GraphError::MissingNode(id))
    }

    /// Set the state of a node.
    pub fn set_state(&mut self, id: NodeId, state: TaskState) -> Result<(), GraphError> {
        let node = self.node_mut(id).ok_or(GraphError::MissingNode(id))?;
        node.state = state;
        Ok(())
    }

    /// Add a dependency edge: `dependency` must finish before `dependent`
    /// can run.
    ///
    /// The graph must remain acyclic; adding an edge that would close a
    /// cycle is rejected.
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

    /// Direct dependencies of a node (what it waits for).
    pub fn deps(&self, id: NodeId) -> Result<&[NodeId], GraphError> {
        self.deps
            .get(id.0)
            .map(Vec::as_slice)
            .ok_or(GraphError::MissingNode(id))
    }

    /// Direct dependents of a node (what waits for it).
    pub fn dependents(&self, id: NodeId) -> Result<&[NodeId], GraphError> {
        self.dependents
            .get(id.0)
            .map(Vec::as_slice)
            .ok_or(GraphError::MissingNode(id))
    }

    /// Whether every dependency of the node has finished successfully
    /// ([`TaskState::Succeeded`], [`TaskState::Skipped`] or
    /// [`TaskState::Cached`]). Nodes without dependencies are ready.
    pub fn is_ready(&self, id: NodeId) -> Result<bool, GraphError> {
        Ok(self
            .deps(id)?
            .iter()
            .all(|dep| self.state(*dep).is_ok_and(TaskState::is_successful)))
    }

    /// All nodes that are ready to run right now, in node order.
    pub fn ready_nodes(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|node| {
                node.state == TaskState::Pending && self.is_ready(node.id).unwrap_or(false)
            })
            .map(|node| node.id)
            .collect()
    }

    /// Whether the node has at least one dependency that failed or was
    /// cancelled and has not been cancelled itself.
    pub fn is_blocked(&self, id: NodeId) -> Result<bool, GraphError> {
        Ok(self
            .deps(id)?
            .iter()
            .any(|dep| self.state(*dep).is_ok_and(TaskState::is_unsuccessful)))
    }

    /// Mark a node as failed and cancel every transitive dependent.
    ///
    /// Dependents are set to [`TaskState::Cancelled`]: they did not fail
    /// themselves, but they can never run because something they need
    /// failed.
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

    /// Nodes in an order where every node comes after all of its
    /// dependencies. Deterministic: ties are broken by node ID (insertion
    /// order).
    ///
    /// The graph is guaranteed acyclic by [`Self::add_dependency`], so the
    /// visit always completes.
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

    /// Partition nodes into topological levels: level 0 contains nodes with
    /// no dependencies, and each later level contains nodes whose last
    /// dependency finished in the previous level. Within a level, nodes are
    /// ordered by ID.
    ///
    /// This is the classic layer view of the graph as presented in `forge
    /// build` output.
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

    /// Map every node payload through `f`, preserving node order, edge
    /// structure, and states. This is how a backend turns a `BuildGraph<A>`
    /// into a `BuildGraph<B>` (e.g. package graph into task graph).
    ///
    /// `f` is called as `f(id, payload)` so callers can index auxiliary
    /// per-node data (fingerprints, etc.) keyed by `NodeId`.
    pub fn map_nodes<R>(&self, mut f: impl FnMut(NodeId, &T) -> R) -> BuildGraph<R> {
        let mut out = BuildGraph::new();
        for node in &self.nodes {
            out.add_node(f(node.id, &node.payload));
        }
        for index in 0..self.nodes.len() {
            let id = NodeId(index);
            if let Ok(deps) = self.deps(id) {
                for dep in deps {
                    // `add_node` assigns sequential IDs in the same order,
                    // so original IDs are valid in the new graph.
                    out.add_dependency(*dep, id)
                        .expect("mapped graph inherits the DAG structure");
                }
            }
        }
        out
    }

    /// Reset every node to [`TaskState::Pending`].
    ///
    /// The scheduler calls this on entry so a graph can be run more than
    /// once (e.g. a warm rebuild in tests) with no state left over from a
    /// previous run.
    pub fn reset_states(&mut self) {
        for node in &mut self.nodes {
            node.state = TaskState::Pending;
        }
    }

    /// Structural sanity check: no missing endpoints, no self-loops, and the
    /// graph is acyclic.
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
            // Defensive: insertion-time cycle detection should make this
            // unreachable.
            return Err(GraphError::Cycle {
                dependency: NodeId(0),
                dependent: NodeId(0),
            });
        }
        Ok(())
    }

    /// Whether `from` can reach `to` by following dependency edges.
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
        // adding a parallel edge that does not close a cycle is fine
        graph
            .add_dependency(NodeId(0), NodeId(2))
            .expect("parallel edges are allowed");
        // ... but the reverse edge would close the cycle 0 -> 2 -> 0
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
        // diamond: 0 -> 2, 1 -> 2, 2 -> 3
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
        // add an independent node last in insertion order
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
        // 0,1 -> 2 -> 3; 4 independent
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

        // untouched: sources are ready, others are not
        assert!(!graph.is_ready(NodeId(2)).unwrap());
        assert_eq!(graph.ready_nodes(), vec![NodeId(0)]);

        graph.set_state(NodeId(0), TaskState::Succeeded).unwrap();
        assert!(graph.is_ready(NodeId(1)).unwrap());
        assert!(!graph.is_ready(NodeId(2)).unwrap());
        assert_eq!(graph.ready_nodes(), vec![NodeId(1)]);

        // a failed dependency never makes its dependent ready
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
