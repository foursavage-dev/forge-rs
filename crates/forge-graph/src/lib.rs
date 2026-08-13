//! # forge-graph
//!
//! The Forge build graph: a directed acyclic graph of build tasks with
//! dependency edges, task states, topological ordering and failure
//! propagation.
//!
//! The graph is generic over its node payload (`BuildGraph<T>`), so language
//! backends can attach their own task descriptions. Task-state semantics are
//! defined once here so the scheduler, executor and future cache layers share
//! exactly the same vocabulary.
//!
//! **Current status (milestone 3):** the graph data structure, states,
//! dependency tracking and topological ordering are implemented and unit
//! tested. The scheduler that consumes this graph is implemented in
//! `forge-scheduler`, and task executors live in `forge-executor`.

#![forbid(unsafe_code)]

pub mod error;
pub mod graph;
pub mod state;

pub use error::GraphError;
pub use graph::{BuildGraph, Node, NodeId};
pub use state::TaskState;
