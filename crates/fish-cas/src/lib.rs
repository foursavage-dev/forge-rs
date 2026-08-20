#![forbid(unsafe_code)]

pub mod artifact;
pub mod backend;
pub mod chunking;
pub mod compression;
pub mod error;
pub mod gc;
pub mod reflink;
pub mod storage;

pub use artifact::{Artifact, ArtifactHash, ArtifactMetadata};
pub use backend::{CasBackend, LocalCasBackend, RemoteCasBackend};
pub use chunking::{Chunk, ChunkManifest, FastCdcChunker};
pub use compression::{CompressionAlgorithm, CompressionLevel};
pub use error::{CasError, Result};
pub use gc::{CasGarbageCollector, CasGcConfig};
pub use reflink::{ReflinkMode, reflink_or_copy};
pub use storage::{CasStorage, CasStorageConfig, CleanupPolicy};
