use crate::graph::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphError {
    #[error("no node with id `{0}` in the graph")]
    MissingNode(NodeId),

    #[error("self-dependency is not allowed (`{0}` depends on itself)")]
    SelfDependency(NodeId),

    #[error("edge `{dependency} -> {dependent}` would create a dependency cycle")]
    Cycle {
        dependency: NodeId,

        dependent: NodeId,
    },
}
