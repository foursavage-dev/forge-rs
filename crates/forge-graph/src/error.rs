//! Errors produced by the build graph.

use crate::graph::NodeId;

/// Errors produced by [`crate::BuildGraph`] operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphError {
    /// A node ID does not exist in the graph.
    #[error("no node with id `{0}` in the graph")]
    MissingNode(NodeId),

    /// A node cannot depend on itself.
    #[error("self-dependency is not allowed (`{0}` depends on itself)")]
    SelfDependency(NodeId),

    /// Adding the edge would close a dependency cycle.
    #[error("edge `{dependency} -> {dependent}` would create a dependency cycle")]
    Cycle {
        /// The node acting as a dependency.
        dependency: NodeId,
        /// The node depending on `dependency`.
        dependent: NodeId,
    },
}
