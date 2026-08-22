#![forbid(unsafe_code)]

use crate::artifact::{Artifact, ArtifactHash, ArtifactMetadata};
use crate::backend::{CasBackend, CasStats, LocalCasBackend};
use crate::compression::CompressionAlgorithm;
use crate::error::{CasError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for CAS storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasStorageConfig {
    /// Storage backend type
    pub backend: CasBackendType,
    /// Local storage path (for local backend)
    pub local_path: Option<PathBuf>,
    /// Compression algorithm
    pub compression: CompressionAlgorithm,
    /// Maximum storage size in bytes (0 = unlimited)
    pub max_size_bytes: u64,
    /// Remote storage configuration
    pub remote: Option<RemoteConfig>,
}

impl CasStorageConfig {
    /// Create a local-only CAS configuration
    pub fn local(path: impl Into<PathBuf>) -> Self {
        Self {
            backend: CasBackendType::Local,
            local_path: Some(path.into()),
            compression: CompressionAlgorithm::Zstd,
            max_size_bytes: 0,
            remote: None,
        }
    }

    /// Create a remote-only CAS configuration
    pub fn remote(remote_config: RemoteConfig) -> Self {
        Self {
            backend: CasBackendType::Remote,
            local_path: None,
            compression: CompressionAlgorithm::Zstd,
            max_size_bytes: 0,
            remote: Some(remote_config),
        }
    }

    /// Create a hybrid configuration (local + remote)
    pub fn hybrid(local_path: impl Into<PathBuf>, remote_config: RemoteConfig) -> Self {
        Self {
            backend: CasBackendType::Hybrid,
            local_path: Some(local_path.into()),
            compression: CompressionAlgorithm::Zstd,
            max_size_bytes: 0,
            remote: Some(remote_config),
        }
    }

    /// Set compression algorithm
    pub fn with_compression(mut self, compression: CompressionAlgorithm) -> Self {
        self.compression = compression;
        self
    }

    /// Set maximum storage size
    pub fn with_max_size(mut self, max_size_bytes: u64) -> Self {
        self.max_size_bytes = max_size_bytes;
        self
    }
}

impl Default for CasStorageConfig {
    fn default() -> Self {
        Self::local(".fish/cas")
    }
}

/// CAS backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CasBackendType {
    Local,
    Remote,
    Hybrid,
}

/// Remote storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    /// Remote backend type
    pub backend_type: RemoteBackendType,
    /// Endpoint URL
    pub endpoint: String,
    /// Authentication credentials
    pub auth: Option<AuthConfig>,
    /// Bucket/container name
    pub bucket: String,
    /// Region (for cloud providers)
    pub region: Option<String>,
}

/// Remote backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteBackendType {
    S3,
    GCS,
    Azure,
    Custom,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Access key ID
    pub access_key: Option<String>,
    /// Secret access key
    pub secret_key: Option<String>,
    /// Authentication token
    pub token: Option<String>,
}

/// Main CAS storage interface
pub struct CasStorage {
    backend: Box<dyn CasBackend>,
    config: CasStorageConfig,
}

impl CasStorage {
    /// Create a new CAS storage instance
    pub async fn new(config: CasStorageConfig) -> Result<Self> {
        let backend: Box<dyn CasBackend> = match config.backend {
            CasBackendType::Local => {
                let local_path = config.local_path.as_ref().ok_or_else(|| {
                    CasError::Config("Local path required for local backend".to_string())
                })?;

                Box::new(LocalCasBackend::new(
                    local_path.to_path_buf(),
                    config.compression,
                )?)
            }
            CasBackendType::Remote => {
                #[cfg(feature = "remote")]
                {
                    let remote_config = config.remote.clone().ok_or_else(|| {
                        CasError::Config("Remote config required for remote backend".to_string())
                    })?;

                    Box::new(
                        crate::backend::RemoteCasBackendImpl::new(
                            remote_config,
                            config.compression,
                        )
                        .await?,
                    )
                }

                #[cfg(not(feature = "remote"))]
                {
                    return Err(CasError::Config(
                        "Remote storage requires 'remote' feature".to_string(),
                    ));
                }
            }
            CasBackendType::Hybrid => {
                #[cfg(feature = "remote")]
                {
                    let local_path = config.local_path.clone().ok_or_else(|| {
                        CasError::Config("Local path required for hybrid backend".to_string())
                    })?;

                    let remote_config = config.remote.clone().ok_or_else(|| {
                        CasError::Config("Remote config required for hybrid backend".to_string())
                    })?;

                    Box::new(crate::backend::HybridCasBackend::new(
                        LocalCasBackend::new(local_path, config.compression)?,
                        crate::backend::RemoteCasBackendImpl::new(
                            remote_config,
                            config.compression,
                        )
                        .await?,
                    ))
                }

                #[cfg(not(feature = "remote"))]
                {
                    return Err(CasError::Config(
                        "Remote storage requires 'remote' feature".to_string(),
                    ));
                }
            }
        };

        Ok(Self { backend, config })
    }

    /// Store an artifact in CAS
    pub async fn store(&self, artifact: &Artifact) -> Result<()> {
        // Check quota
        if self.config.max_size_bytes > 0 {
            let stats = self.backend.stats().await?;
            if stats.total_bytes.saturating_add(artifact.size()) > self.config.max_size_bytes {
                return Err(CasError::QuotaExceeded(format!(
                    "Storage quota exceeded: {} + {} > {}",
                    stats.total_bytes,
                    artifact.size(),
                    self.config.max_size_bytes
                )));
            }
        }

        self.backend.store(artifact).await
    }

    /// Retrieve an artifact by hash
    pub async fn retrieve(&self, hash: &ArtifactHash) -> Result<Artifact> {
        self.backend.retrieve(hash).await
    }

    /// Retrieve artifact metadata without reading or decompressing its payload.
    pub async fn metadata(&self, hash: &ArtifactHash) -> Result<ArtifactMetadata> {
        self.backend.metadata(hash).await
    }

    /// Check if an artifact exists
    pub async fn exists(&self, hash: &ArtifactHash) -> Result<bool> {
        self.backend.exists(hash).await
    }

    /// Delete an artifact
    pub async fn delete(&self, hash: &ArtifactHash) -> Result<()> {
        self.backend.delete(hash).await
    }

    /// List all artifacts
    pub async fn list(&self) -> Result<Vec<ArtifactHash>> {
        self.backend.list().await
    }

    /// Get storage statistics
    pub async fn stats(&self) -> Result<CasStats> {
        self.backend.stats().await
    }

    /// Clean up old artifacts based on policy
    pub async fn cleanup(&self, policy: CleanupPolicy) -> Result<CleanupResult> {
        let hashes = self.list().await?;
        let mut removed_count = 0;
        let mut freed_bytes = 0u64;

        // Cleanup decisions use timestamps and tags. Loading the full artifact
        // here used to read and decompress every object in the cache.
        let mut entries = Vec::new();
        for hash in hashes {
            if let Ok(metadata) = self.metadata(&hash).await {
                entries.push((hash, metadata));
            }
        }

        let removals: Vec<_> = match policy {
            CleanupPolicy::KeepMostRecent(keep) => {
                entries.sort_by_key(|(_, right)| std::cmp::Reverse(right.timestamp));
                entries.into_iter().skip(keep).collect()
            }
            policy => entries
                .into_iter()
                .filter(|(_, metadata)| policy.should_remove_metadata(metadata))
                .collect(),
        };

        for (hash, metadata) in removals {
            self.delete(&hash).await?;
            removed_count += 1;
            freed_bytes += metadata.size;
        }

        Ok(CleanupResult {
            removed_count,
            freed_bytes,
        })
    }

    /// Get the storage configuration
    pub fn config(&self) -> &CasStorageConfig {
        &self.config
    }
}

/// Cleanup policy for CAS storage
#[derive(Debug, Clone)]
pub enum CleanupPolicy {
    /// Remove artifacts older than specified duration
    OlderThan(std::time::Duration),
    /// Remove artifacts that haven't been accessed in specified duration
    NotAccessedIn(std::time::Duration),
    /// Remove artifacts matching specific tags
    RemoveTags(Vec<String>),
    /// Keep only the N most recent artifacts
    KeepMostRecent(usize),
}

impl CleanupPolicy {
    fn should_remove_metadata(&self, metadata: &ArtifactMetadata) -> bool {
        match self {
            Self::OlderThan(duration) => {
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                current_time - metadata.timestamp > duration.as_secs() as i64
            }
            Self::NotAccessedIn(duration) => {
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                let threshold = duration.as_secs() as i64;
                // Prefer the last-access timestamp; fall back to the store
                // time for records written before access tracking existed.
                let last = metadata.last_accessed.unwrap_or(metadata.timestamp);
                current_time - last > threshold
            }
            Self::RemoveTags(tags) => tags.iter().any(|tag| metadata.tags.contains(tag)),
            Self::KeepMostRecent(_) => false,
        }
    }

    pub fn should_remove(&self, artifact: &Artifact) -> bool {
        self.should_remove_metadata(&artifact.metadata)
    }
}

/// Result of a cleanup operation
#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub removed_count: usize,
    pub freed_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cas_storage() {
        let temp_dir = tempdir().unwrap();
        let config = CasStorageConfig::local(temp_dir.path().join("cas"));
        let storage = CasStorage::new(config).await.unwrap();

        let artifact = Artifact::from_bytes(
            b"test artifact data".to_vec(),
            "binary".to_string(),
            "test".to_string(),
        )
        .unwrap();

        // Store artifact
        storage.store(&artifact).await.unwrap();

        // Check existence
        assert!(storage.exists(artifact.hash()).await.unwrap());

        // Retrieve artifact
        let retrieved = storage.retrieve(artifact.hash()).await.unwrap();
        assert_eq!(retrieved.data(), artifact.data());

        // Get stats
        let stats = storage.stats().await.unwrap();
        assert_eq!(stats.artifact_count, 1);
        assert_eq!(stats.total_bytes, artifact.size());
    }

    #[tokio::test]
    async fn test_cas_config() {
        let config = CasStorageConfig::local("./test_cas")
            .with_compression(CompressionAlgorithm::ZstdMax)
            .with_max_size(1024 * 1024 * 1024); // 1GB

        assert_eq!(config.backend, CasBackendType::Local);
        assert_eq!(config.compression, CompressionAlgorithm::ZstdMax);
        assert_eq!(config.max_size_bytes, 1024 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_cleanup_policy() {
        let artifact = Artifact::from_bytes(
            b"test data".to_vec(),
            "binary".to_string(),
            "test".to_string(),
        )
        .unwrap();

        let old_artifact = {
            let mut metadata = artifact.metadata.clone();
            metadata.timestamp = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64)
                - (10 * 24 * 60 * 60); // 10 days ago
            Artifact {
                metadata,
                data: artifact.data.clone(),
                original_path: None,
            }
        };

        let policy = CleanupPolicy::OlderThan(std::time::Duration::from_secs(60 * 60 * 24 * 7)); // 7 days
        assert!(!policy.should_remove(&artifact)); // Recent artifact
        assert!(policy.should_remove(&old_artifact)); // Old artifact

        // NotAccessedIn is driven by last_accessed when present, falling back
        // to the store timestamp for legacy records.
        let not_accessed =
            CleanupPolicy::NotAccessedIn(std::time::Duration::from_secs(60 * 60 * 24 * 7));

        let mut untouched = artifact.clone();
        untouched.metadata.last_accessed = None;
        assert!(
            !not_accessed.should_remove(&untouched),
            "fresh record must survive"
        );

        let mut stale = artifact.clone();
        stale.metadata.last_accessed = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
                - 10 * 24 * 60 * 60,
        );
        assert!(
            not_accessed.should_remove(&stale),
            "long-untouched record must be removed"
        );

        let mut legacy = artifact.clone();
        legacy.metadata.last_accessed = None;
        legacy.metadata.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - 10 * 24 * 60 * 60;
        assert!(
            not_accessed.should_remove(&legacy),
            "legacy records without last_accessed fall back to their store time"
        );

        // Test RemoveTags policy
        let mut tagged_artifact = artifact.clone();
        tagged_artifact
            .metadata
            .tags
            .push("cleanup-test".to_string());
        let tag_policy = CleanupPolicy::RemoveTags(vec!["cleanup-test".to_string()]);
        assert!(tag_policy.should_remove(&tagged_artifact));
        assert!(!tag_policy.should_remove(&artifact));
    }

    #[tokio::test]
    async fn cleanup_uses_metadata_without_loading_artifact_payloads() {
        let temp_dir = tempdir().unwrap();
        let storage = CasStorage::new(
            CasStorageConfig::local(temp_dir.path().join("cas"))
                .with_compression(CompressionAlgorithm::Zstd),
        )
        .await
        .unwrap();
        let mut artifact =
            Artifact::from_bytes(vec![0; 8 * 1024], "binary".to_string(), "test".to_string())
                .unwrap();
        artifact.metadata.tags.push("expired".to_string());
        let hash = artifact.hash().clone();
        storage.store(&artifact).await.unwrap();

        // A damaged compressed payload cannot be retrieved. Cleanup can still
        // delete it because the policy only needs its metadata.
        let data_path = temp_dir
            .path()
            .join("cas")
            .join(&hash.as_str()[..2])
            .join(&hash.as_str()[2..]);
        tokio::fs::write(data_path, b"damaged payload")
            .await
            .unwrap();

        let result = storage
            .cleanup(CleanupPolicy::RemoveTags(vec!["expired".to_string()]))
            .await
            .unwrap();
        assert_eq!(result.removed_count, 1);
        assert!(!storage.exists(&hash).await.unwrap());
    }

    #[tokio::test]
    async fn test_high_concurrency_cas_stress() {
        let temp_dir = tempdir().unwrap();
        let storage = std::sync::Arc::new(
            CasStorage::new(
                CasStorageConfig::local(temp_dir.path().join("cas"))
                    .with_compression(CompressionAlgorithm::Zstd),
            )
            .await
            .unwrap(),
        );

        let mut handles = Vec::new();
        for worker_id in 0..16 {
            let storage = std::sync::Arc::clone(&storage);
            handles.push(tokio::spawn(async move {
                for item in 0..8 {
                    let payload = format!("worker_{worker_id}_payload_{item}").into_bytes();
                    let artifact = Artifact::from_bytes(
                        payload.clone(),
                        "text/plain".to_string(),
                        format!("worker_{worker_id}"),
                    )
                    .unwrap();

                    let hash = artifact.hash().clone();
                    storage.store(&artifact).await.unwrap();
                    assert!(storage.exists(&hash).await.unwrap());

                    let fetched = storage.retrieve(&hash).await.unwrap();
                    assert_eq!(fetched.data, payload);
                }
            }));
        }

        for h in handles {
            h.await.unwrap();
        }
    }
}
