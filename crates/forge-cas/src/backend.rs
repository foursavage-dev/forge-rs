#![forbid(unsafe_code)]

use crate::artifact::{Artifact, ArtifactHash, ArtifactMetadata};
use crate::compression::CompressionAlgorithm;
use crate::error::{CasError, Result};
use async_trait::async_trait;
use std::str::FromStr;

/// Trait for CAS backend implementations
#[async_trait]
pub trait CasBackend: Send + Sync {
    /// Store an artifact in the CAS
    async fn store(&self, artifact: &Artifact) -> Result<()>;

    /// Retrieve an artifact by hash
    async fn retrieve(&self, hash: &ArtifactHash) -> Result<Artifact>;

    /// Retrieve artifact metadata without reading the artifact payload.
    async fn metadata(&self, hash: &ArtifactHash) -> Result<ArtifactMetadata>;

    /// Check if an artifact exists
    async fn exists(&self, hash: &ArtifactHash) -> Result<bool>;

    /// Delete an artifact
    async fn delete(&self, hash: &ArtifactHash) -> Result<()>;

    /// List all artifacts
    async fn list(&self) -> Result<Vec<ArtifactHash>>;

    /// Get storage statistics
    async fn stats(&self) -> Result<CasStats>;
}

/// Storage statistics for a CAS backend
#[derive(Debug, Clone)]
pub struct CasStats {
    /// Total number of artifacts
    pub artifact_count: usize,
    /// Total storage used in bytes
    pub total_bytes: u64,
    /// Total storage used after compression in bytes
    pub compressed_bytes: u64,
    /// Storage backend type
    pub backend_type: String,
}

/// Local file-system based CAS backend
pub struct LocalCasBackend {
    base_path: std::path::PathBuf,
    compression: crate::compression::CompressionAlgorithm,
}

impl LocalCasBackend {
    /// Create a new local CAS backend
    pub fn new(
        base_path: std::path::PathBuf,
        compression: crate::compression::CompressionAlgorithm,
    ) -> Result<Self> {
        std::fs::create_dir_all(&base_path).map_err(CasError::Io)?;

        Ok(Self {
            base_path,
            compression,
        })
    }

    /// Get the file path for a given hash
    fn hash_path(&self, hash: &ArtifactHash) -> Result<std::path::PathBuf> {
        let hash_str = hash.as_str();
        validate_hash(hash_str)?;
        // Use first 2 characters as directory for better file system performance
        let dir = &hash_str[..2];
        let filename = &hash_str[2..];
        Ok(self.base_path.join(dir).join(filename))
    }

    /// Get the metadata path for a given hash
    fn metadata_path(&self, hash: &ArtifactHash) -> Result<std::path::PathBuf> {
        let mut path = self.hash_path(hash)?;
        path.set_extension("meta");
        Ok(path)
    }
}

/// CAS objects are addressed by a 32-byte BLAKE3 digest encoded as lowercase
/// hexadecimal. Validating at the storage boundary prevents an untrusted hash
/// from escaping the CAS root through path components.
fn validate_hash(hash: &str) -> Result<()> {
    if hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(CasError::Config(format!(
            "invalid artifact hash `{hash}`; expected a 64-character hexadecimal BLAKE3 digest"
        )))
    }
}

#[async_trait]
impl CasBackend for LocalCasBackend {
    async fn store(&self, artifact: &Artifact) -> Result<()> {
        let hash = artifact.hash();
        let computed_hash = ArtifactHash::from_bytes(artifact.data())?;
        if &computed_hash != hash {
            return Err(CasError::Hash(format!(
                "artifact data does not match declared hash `{hash}`"
            )));
        }
        let data_path = self.hash_path(hash)?;
        let metadata_path = self.metadata_path(hash)?;

        // Create parent directory
        if let Some(parent) = data_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(CasError::Io)?;
        }

        // Compress data if configured
        let (data_to_store, compressed_size) =
            if self.compression != crate::compression::CompressionAlgorithm::None {
                let compressed = crate::compression::compress(artifact.data(), self.compression)?;
                let compressed_size = compressed.len() as u64;
                (compressed, Some(compressed_size))
            } else {
                (artifact.data().to_vec(), None)
            };

        let tmp_data = data_path.with_extension(format!("tmp.{}", std::process::id()));
        tokio::fs::write(&tmp_data, &data_to_store)
            .await
            .map_err(CasError::Io)?;
        let _ = tokio::fs::rename(&tmp_data, &data_path).await;

        let mut metadata = artifact.metadata.clone();
        if let Some(size) = compressed_size {
            metadata = metadata.with_compression(size, self.compression.as_str().to_string());
        }

        let metadata_json =
            serde_json::to_string(&metadata).map_err(|e| CasError::Serialization(e.to_string()))?;

        let tmp_meta = metadata_path.with_extension(format!("meta.tmp.{}", std::process::id()));
        tokio::fs::write(&tmp_meta, metadata_json)
            .await
            .map_err(CasError::Io)?;
        let _ = tokio::fs::rename(&tmp_meta, &metadata_path).await;

        Ok(())
    }

    async fn retrieve(&self, hash: &ArtifactHash) -> Result<Artifact> {
        let data_path = self.hash_path(hash)?;
        let metadata_path = self.metadata_path(hash)?;

        // Check if files exist
        if !data_path.exists() || !metadata_path.exists() {
            return Err(CasError::ArtifactNotFound(hash.to_string()));
        }

        // Read metadata
        let metadata_json = tokio::fs::read_to_string(&metadata_path)
            .await
            .map_err(CasError::Io)?;

        let metadata: crate::artifact::ArtifactMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| CasError::Serialization(e.to_string()))?;

        // Read and decompress data
        let data = tokio::fs::read(&data_path).await.map_err(CasError::Io)?;

        let decompressed_data = if metadata.compression.is_some() {
            let algorithm = metadata
                .compression
                .as_ref()
                .and_then(|alg| CompressionAlgorithm::from_str(alg).ok())
                .unwrap_or(CompressionAlgorithm::None);

            crate::compression::decompress(&data, algorithm)?
        } else {
            data
        };

        let computed_hash = ArtifactHash::from_bytes(&decompressed_data)?;
        if &computed_hash != hash || &metadata.hash != hash {
            return Err(CasError::Hash(format!(
                "stored artifact failed integrity verification for `{hash}`"
            )));
        }

        Ok(Artifact {
            metadata,
            data: decompressed_data,
            original_path: None,
        })
    }

    async fn metadata(&self, hash: &ArtifactHash) -> Result<ArtifactMetadata> {
        let metadata_path = self.metadata_path(hash)?;
        let metadata_json = tokio::fs::read_to_string(&metadata_path)
            .await
            .map_err(|error| match error.kind() {
                std::io::ErrorKind::NotFound => CasError::ArtifactNotFound(hash.to_string()),
                _ => CasError::Io(error),
            })?;
        let metadata: ArtifactMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| CasError::Serialization(e.to_string()))?;
        if &metadata.hash != hash {
            return Err(CasError::Hash(format!(
                "stored metadata failed integrity verification for `{hash}`"
            )));
        }
        Ok(metadata)
    }

    async fn exists(&self, hash: &ArtifactHash) -> Result<bool> {
        let data_path = self.hash_path(hash)?;
        let metadata_path = self.metadata_path(hash)?;

        Ok(data_path.exists() && metadata_path.exists())
    }

    async fn delete(&self, hash: &ArtifactHash) -> Result<()> {
        let data_path = self.hash_path(hash)?;
        let metadata_path = self.metadata_path(hash)?;

        if data_path.exists() {
            tokio::fs::remove_file(&data_path)
                .await
                .map_err(CasError::Io)?;
        }

        if metadata_path.exists() {
            tokio::fs::remove_file(&metadata_path)
                .await
                .map_err(CasError::Io)?;
        }

        Ok(())
    }

    async fn list(&self) -> Result<Vec<ArtifactHash>> {
        let mut hashes = Vec::new();

        let mut entries = tokio::fs::read_dir(&self.base_path)
            .await
            .map_err(CasError::Io)?;

        while let Some(entry) = entries.next_entry().await.map_err(CasError::Io)? {
            let path = entry.path();
            if path.is_dir() {
                let mut sub_entries = tokio::fs::read_dir(&path).await.map_err(CasError::Io)?;

                while let Some(sub_entry) = sub_entries.next_entry().await.map_err(CasError::Io)? {
                    let sub_path = sub_entry.path();
                    if sub_path.is_file()
                        && sub_path.extension().map(|e| e != "meta").unwrap_or(true)
                        && let Some(hash_str) = sub_path.file_stem().and_then(|s| s.to_str())
                    {
                        let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                        let full_hash = format!("{}{}", dir_name, hash_str);
                        if validate_hash(&full_hash).is_ok() {
                            hashes.push(ArtifactHash::new(full_hash));
                        }
                    }
                }
            }
        }

        Ok(hashes)
    }

    async fn stats(&self) -> Result<CasStats> {
        let hashes = self.list().await?;
        let mut total_bytes = 0u64;
        let mut compressed_bytes = 0u64;
        let mut artifact_count = 0usize;

        for hash in &hashes {
            // Statistics are requested on every cache inspection and quota
            // check. Reading and decompressing each object here makes those
            // operations scale with the full artifact payload. Metadata holds
            // the original size, while filesystem metadata provides the exact
            // compressed on-disk size without loading the object.
            let data_path = self.hash_path(hash)?;
            let metadata_path = self.metadata_path(hash)?;
            let (data_info, metadata_json) = match (
                tokio::fs::metadata(&data_path).await,
                tokio::fs::read_to_string(&metadata_path).await,
            ) {
                (Ok(data_info), Ok(metadata_json)) => (data_info, metadata_json),
                // Match the previous best-effort behaviour for partial or
                // corrupt entries, but do not count them as healthy artifacts.
                _ => continue,
            };
            let metadata: crate::artifact::ArtifactMetadata =
                match serde_json::from_str::<crate::artifact::ArtifactMetadata>(&metadata_json) {
                    Ok(metadata) if &metadata.hash == hash => metadata,
                    _ => continue,
                };

            artifact_count += 1;
            total_bytes += metadata.size;
            compressed_bytes += data_info.len();
        }

        Ok(CasStats {
            artifact_count,
            total_bytes,
            compressed_bytes,
            backend_type: "local".to_string(),
        })
    }
}

/// Remote CAS backend (abstract, to be implemented by S3, GCS, etc.)
#[async_trait]
pub trait RemoteCasBackend: CasBackend {
    /// Upload artifact to remote storage
    async fn upload(&self, artifact: &Artifact) -> Result<()>;

    /// Download artifact from remote storage
    async fn download(&self, hash: &ArtifactHash) -> Result<Artifact>;

    /// Check remote connection
    async fn health_check(&self) -> Result<bool>;
}

/// Placeholder implementation for remote CAS backend
#[cfg(feature = "remote")]
pub struct RemoteCasBackendImpl {
    _config: crate::storage::RemoteConfig,
    _compression: crate::compression::CompressionAlgorithm,
}

#[cfg(feature = "remote")]
impl RemoteCasBackendImpl {
    pub async fn new(
        config: crate::storage::RemoteConfig,
        compression: crate::compression::CompressionAlgorithm,
    ) -> Result<Self> {
        // TODO: Implement actual remote backend initialization
        Ok(Self {
            _config: config,
            _compression: compression,
        })
    }
}

#[cfg(feature = "remote")]
#[async_trait]
impl CasBackend for RemoteCasBackendImpl {
    async fn store(&self, artifact: &Artifact) -> Result<()> {
        self.upload(artifact).await
    }

    async fn retrieve(&self, hash: &ArtifactHash) -> Result<Artifact> {
        self.download(hash).await
    }

    async fn metadata(&self, _hash: &ArtifactHash) -> Result<ArtifactMetadata> {
        Err(CasError::BackendError(
            "Remote metadata lookup not yet implemented".to_string(),
        ))
    }

    async fn exists(&self, _hash: &ArtifactHash) -> Result<bool> {
        // TODO: Implement existence check
        Ok(false)
    }

    async fn delete(&self, _hash: &ArtifactHash) -> Result<()> {
        // TODO: Implement deletion
        Ok(())
    }

    async fn list(&self) -> Result<Vec<ArtifactHash>> {
        // TODO: Implement listing
        Ok(Vec::new())
    }

    async fn stats(&self) -> Result<CasStats> {
        Ok(CasStats {
            artifact_count: 0,
            total_bytes: 0,
            compressed_bytes: 0,
            backend_type: "remote".to_string(),
        })
    }
}

#[cfg(feature = "remote")]
#[async_trait]
impl RemoteCasBackend for RemoteCasBackendImpl {
    async fn upload(&self, _artifact: &Artifact) -> Result<()> {
        // TODO: Implement actual upload
        Err(CasError::BackendError(
            "Remote upload not yet implemented".to_string(),
        ))
    }

    async fn download(&self, _hash: &ArtifactHash) -> Result<Artifact> {
        // TODO: Implement actual download
        Err(CasError::BackendError(
            "Remote download not yet implemented".to_string(),
        ))
    }

    async fn health_check(&self) -> Result<bool> {
        // TODO: Implement health check
        Ok(true)
    }
}

/// Hybrid CAS backend that uses local as cache and remote as backing store
#[cfg(feature = "remote")]
pub struct HybridCasBackend {
    local: LocalCasBackend,
    remote: RemoteCasBackendImpl,
}

#[cfg(feature = "remote")]
impl HybridCasBackend {
    pub fn new(local: LocalCasBackend, remote: RemoteCasBackendImpl) -> Self {
        Self { local, remote }
    }
}

#[cfg(feature = "remote")]
#[async_trait]
impl CasBackend for HybridCasBackend {
    async fn store(&self, artifact: &Artifact) -> Result<()> {
        // Store locally first
        self.local.store(artifact).await?;
        // Then upload to remote
        self.remote.upload(artifact).await
    }

    async fn retrieve(&self, hash: &ArtifactHash) -> Result<Artifact> {
        // Try local first
        if self.local.exists(hash).await? {
            return self.local.retrieve(hash).await;
        }
        // Fall back to remote
        let artifact = self.remote.download(hash).await?;
        // Cache locally
        self.local.store(&artifact).await?;
        Ok(artifact)
    }

    async fn metadata(&self, hash: &ArtifactHash) -> Result<ArtifactMetadata> {
        if self.local.exists(hash).await? {
            return self.local.metadata(hash).await;
        }
        self.remote.metadata(hash).await
    }

    async fn exists(&self, hash: &ArtifactHash) -> Result<bool> {
        if self.local.exists(hash).await? {
            return Ok(true);
        }
        self.remote.exists(hash).await
    }

    async fn delete(&self, hash: &ArtifactHash) -> Result<()> {
        self.local.delete(hash).await?;
        self.remote.delete(hash).await
    }

    async fn list(&self) -> Result<Vec<ArtifactHash>> {
        // Combine local and remote listings
        let mut local_hashes = self.local.list().await?;
        let remote_hashes = self.remote.list().await?;

        // Deduplicate
        local_hashes.extend(remote_hashes);
        local_hashes.sort();
        local_hashes.dedup();

        Ok(local_hashes)
    }

    async fn stats(&self) -> Result<CasStats> {
        let local_stats = self.local.stats().await?;
        let remote_stats = self.remote.stats().await?;

        Ok(CasStats {
            artifact_count: local_stats.artifact_count + remote_stats.artifact_count,
            total_bytes: local_stats.total_bytes + remote_stats.total_bytes,
            compressed_bytes: local_stats.compressed_bytes + remote_stats.compressed_bytes,
            backend_type: "hybrid".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_local_cas_backend() {
        let temp_dir = tempdir().unwrap();
        let backend = LocalCasBackend::new(
            temp_dir.path().to_path_buf(),
            crate::compression::CompressionAlgorithm::None,
        )
        .unwrap();

        let artifact = Artifact::from_bytes(
            b"test artifact data".to_vec(),
            "binary".to_string(),
            "test".to_string(),
        )
        .unwrap();

        // Store artifact
        backend.store(&artifact).await.unwrap();

        // Check existence
        assert!(backend.exists(artifact.hash()).await.unwrap());

        // Retrieve artifact
        let retrieved = backend.retrieve(artifact.hash()).await.unwrap();
        assert_eq!(retrieved.data(), artifact.data());

        // List artifacts
        let hashes = backend.list().await.unwrap();
        assert_eq!(hashes.len(), 1);
        assert_eq!(hashes[0], *artifact.hash());

        // Delete artifact
        backend.delete(artifact.hash()).await.unwrap();
        assert!(!backend.exists(artifact.hash()).await.unwrap());
    }

    #[tokio::test]
    async fn test_local_cas_with_compression() {
        let temp_dir = tempdir().unwrap();
        let backend = LocalCasBackend::new(
            temp_dir.path().to_path_buf(),
            crate::compression::CompressionAlgorithm::Zstd,
        )
        .unwrap();

        let data = vec![0u8; 10000]; // Highly compressible
        let artifact =
            Artifact::from_bytes(data.clone(), "binary".to_string(), "test".to_string()).unwrap();

        backend.store(&artifact).await.unwrap();

        let retrieved = backend.retrieve(artifact.hash()).await.unwrap();
        assert_eq!(retrieved.data(), data.as_slice());
        assert!(retrieved.compression_ratio().is_some());
        assert!(retrieved.compression_ratio().unwrap() < 1.0);
    }

    #[tokio::test]
    async fn test_cas_stats() {
        let temp_dir = tempdir().unwrap();
        let backend = LocalCasBackend::new(
            temp_dir.path().to_path_buf(),
            crate::compression::CompressionAlgorithm::None,
        )
        .unwrap();

        let artifact1 = Artifact::from_bytes(
            b"test data 1".to_vec(),
            "binary".to_string(),
            "test1".to_string(),
        )
        .unwrap();

        let artifact2 = Artifact::from_bytes(
            b"test data 2".to_vec(),
            "binary".to_string(),
            "test2".to_string(),
        )
        .unwrap();

        backend.store(&artifact1).await.unwrap();
        backend.store(&artifact2).await.unwrap();

        let stats = backend.stats().await.unwrap();
        assert_eq!(stats.artifact_count, 2);
        assert_eq!(stats.total_bytes, artifact1.size() + artifact2.size());
        assert_eq!(stats.backend_type, "local");
    }

    #[tokio::test]
    async fn stats_uses_metadata_without_reading_or_decompressing_objects() {
        let temp_dir = tempdir().unwrap();
        let backend = LocalCasBackend::new(
            temp_dir.path().to_path_buf(),
            crate::compression::CompressionAlgorithm::Zstd,
        )
        .unwrap();
        let data = vec![0; 16 * 1024];
        let artifact =
            Artifact::from_bytes(data.clone(), "binary".to_string(), "test".to_string()).unwrap();
        backend.store(&artifact).await.unwrap();

        // Invalid compressed bytes cannot be retrieved, but computing stats
        // must remain cheap and use the persisted metadata plus file length.
        let data_path = backend.hash_path(artifact.hash()).unwrap();
        let stored_len = tokio::fs::metadata(&data_path).await.unwrap().len();
        tokio::fs::write(&data_path, vec![0xff; stored_len as usize])
            .await
            .unwrap();

        let stats = backend.stats().await.unwrap();
        assert_eq!(stats.artifact_count, 1);
        assert_eq!(stats.total_bytes, data.len() as u64);
        assert_eq!(stats.compressed_bytes, stored_len);
    }

    #[tokio::test]
    async fn rejects_invalid_hashes_without_touching_the_storage_root() {
        let temp_dir = tempdir().unwrap();
        let backend = LocalCasBackend::new(
            temp_dir.path().to_path_buf(),
            crate::compression::CompressionAlgorithm::None,
        )
        .unwrap();
        let invalid = ArtifactHash::new("../../outside".to_string());

        let error = backend.exists(&invalid).await.unwrap_err();
        assert!(matches!(error, CasError::Config(_)));
        assert!(temp_dir.path().read_dir().unwrap().next().is_none());
    }

    #[tokio::test]
    async fn detects_corrupted_artifact_contents() {
        let temp_dir = tempdir().unwrap();
        let backend = LocalCasBackend::new(
            temp_dir.path().to_path_buf(),
            crate::compression::CompressionAlgorithm::None,
        )
        .unwrap();
        let artifact = Artifact::from_bytes(
            b"original content".to_vec(),
            "binary".to_string(),
            "test".to_string(),
        )
        .unwrap();
        backend.store(&artifact).await.unwrap();

        tokio::fs::write(backend.hash_path(artifact.hash()).unwrap(), b"tampered")
            .await
            .unwrap();

        let error = backend.retrieve(artifact.hash()).await.unwrap_err();
        assert!(matches!(error, CasError::Hash(_)));
    }
}
