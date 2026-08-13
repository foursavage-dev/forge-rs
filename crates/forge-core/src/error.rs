//! Structured error types for Forge core.

use std::path::PathBuf;

/// Errors produced by Forge core.
#[derive(Debug, thiserror::Error)]
pub enum ForgeError {
    /// Cargo metadata could not be loaded for the given manifest.
    #[error("failed to load Cargo metadata for `{manifest}`")]
    CargoMetadata {
        /// Manifest path the metadata was requested for.
        manifest: PathBuf,
        /// Underlying Cargo failure.
        #[source]
        source: cargo_metadata::Error,
    },

    /// The manifest path is not valid UTF-8 and cannot be passed to Cargo.
    #[error("manifest path is not valid UTF-8: `{0}`")]
    NonUtf8ManifestPath(PathBuf),

    /// The dependency graph derived from Cargo metadata is invalid.
    #[error("invalid dependency graph derived from Cargo metadata")]
    BuildGraph(#[from] forge_graph::GraphError),
}

/// Convenience result type used across Forge core.
pub type Result<T> = std::result::Result<T, ForgeError>;
