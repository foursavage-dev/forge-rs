//! # forge-core
//!
//! Core library of the Forge build orchestration system.
//!
//! The Forge architecture is layered:
//!
//! ```text
//! Project
//!    ↓
//! Graph
//!    ↓
//! Tasks
//!    ↓
//! Scheduler
//!    ↓
//! Executor
//!    ↓
//! Artifacts
//!    ↓
//! Cache
//! ```
//!
//! Language-specific logic lives behind backend APIs, never in Forge Core.
//!
//! **Current status (milestone 1):** this crate implements Cargo project
//! discovery and metadata loading only. The build graph, scheduler, executor
//! and cache are not implemented yet.

#![forbid(unsafe_code)]

pub mod backend;
pub mod error;
pub mod project;

pub use backend::BuildBackend;
pub use error::{ForgeError, Result};
