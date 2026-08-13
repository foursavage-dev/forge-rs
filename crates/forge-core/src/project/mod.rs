//! Cargo project discovery and metadata integration.
//!
//! Forge consumes official Cargo metadata (`cargo metadata --format-version 1`)
//! rather than parsing `Cargo.toml` itself. Cargo remains responsible for
//! dependency resolution and package management; Forge reads its output.

mod detect;
mod model;

pub use detect::{find_manifest, find_manifest_dir};
pub use model::Project;
