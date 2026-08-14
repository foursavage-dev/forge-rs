#![forbid(unsafe_code)]

pub mod artifact;
pub mod backend;
pub mod compression;
pub mod error;
pub mod reflink;
pub mod storage;

pub use artifact::{Artifact, ArtifactHash, ArtifactMetadata};
pub use backend::{CasBackend, LocalCasBackend, RemoteCasBackend};
pub use compression::{CompressionAlgorithm, CompressionLevel};
pub use error::{CasError, Result};
pub use reflink::{reflink_or_copy, ReflinkMode};
pub use storage::{CasStorage, CasStorageConfig, CleanupPolicy};
