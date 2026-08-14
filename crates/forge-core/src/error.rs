use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ForgeError {
    #[error("failed to load Cargo metadata for `{manifest}`")]
    CargoMetadata {
        manifest: PathBuf,

        #[source]
        source: cargo_metadata::Error,
    },

    #[error("manifest path is not valid UTF-8: `{0}`")]
    NonUtf8ManifestPath(PathBuf),

    #[error("invalid dependency graph derived from Cargo metadata")]
    BuildGraph(#[from] forge_graph::GraphError),
}

pub type Result<T> = std::result::Result<T, ForgeError>;
