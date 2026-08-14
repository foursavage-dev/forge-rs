#![forbid(unsafe_code)]

use crate::error::{CasError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

/// Content-based hash for artifact identification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactHash(String);

impl ArtifactHash {
    /// Create a new hash from raw bytes
    pub fn new(hash: String) -> Self {
        Self(hash)
    }
    
    /// Create hash from file content
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read(path)
            .map_err(|e| CasError::Hash(format!("Failed to read file: {}", e)))?;
        Self::from_bytes(&content)
    }
    
    /// Create hash from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let hash = blake3::hash(data);
        Ok(Self(hash.to_hex().to_string()))
    }
    
    /// Get the hash string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the hash as bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl std::fmt::Display for ArtifactHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata about a stored artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// Hash of the artifact content
    pub hash: ArtifactHash,
    /// Original file size in bytes
    pub size: u64,
    /// Compressed size in bytes (if applicable)
    pub compressed_size: Option<u64>,
    /// Compression algorithm used
    pub compression: Option<String>,
    /// Timestamp when artifact was stored (Unix timestamp in seconds)
    pub timestamp: i64,
    /// Artifact type/category
    pub artifact_type: String,
    /// Source information (e.g., package name, target)
    pub source: String,
    /// Additional metadata
    pub tags: Vec<String>,
}

impl ArtifactMetadata {
    pub fn new(
        hash: ArtifactHash,
        size: u64,
        artifact_type: String,
        source: String,
    ) -> Self {
        Self {
            hash,
            size,
            compressed_size: None,
            compression: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            artifact_type,
            source,
            tags: Vec::new(),
        }
    }
    
    pub fn with_compression(mut self, compressed_size: u64, algorithm: String) -> Self {
        self.compressed_size = Some(compressed_size);
        self.compression = Some(algorithm);
        self
    }
    
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
    
    pub fn compression_ratio(&self) -> Option<f64> {
        self.compressed_size.map(|compressed| compressed as f64 / self.size as f64)
    }
}

/// A build artifact that can be stored in CAS
#[derive(Debug, Clone)]
pub struct Artifact {
    pub metadata: ArtifactMetadata,
    pub data: Vec<u8>,
    pub original_path: Option<PathBuf>,
}

impl Artifact {
    /// Create artifact from file
    pub async fn from_file(path: &Path) -> Result<Self> {
        let data = tokio::fs::read(path)
            .await
            .map_err(CasError::Io)?;
        
        let hash = ArtifactHash::from_bytes(&data)?;
        let size = data.len() as u64;
        
        let artifact_type = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("binary")
            .to_string();
        
        let source = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let metadata = ArtifactMetadata::new(hash, size, artifact_type, source);
        
        Ok(Self {
            metadata,
            data,
            original_path: Some(path.to_path_buf()),
        })
    }
    
    /// Create artifact from bytes
    pub fn from_bytes(data: Vec<u8>, artifact_type: String, source: String) -> Result<Self> {
        let hash = ArtifactHash::from_bytes(&data)?;
        let size = data.len() as u64;
        let metadata = ArtifactMetadata::new(hash, size, artifact_type, source);
        
        Ok(Self {
            metadata,
            data,
            original_path: None,
        })
    }
    
    /// Get the artifact hash
    pub fn hash(&self) -> &ArtifactHash {
        &self.metadata.hash
    }
    
    /// Get artifact data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    /// Get artifact size
    pub fn size(&self) -> u64 {
        self.metadata.size
    }
    
    /// Get compression ratio
    pub fn compression_ratio(&self) -> Option<f64> {
        match (self.metadata.size, self.metadata.compressed_size) {
            (original, Some(compressed)) if original > 0 => {
                Some(compressed as f64 / original as f64)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_artifact_hash() {
        let data = b"test data";
        let hash1 = ArtifactHash::from_bytes(data).unwrap();
        let hash2 = ArtifactHash::from_bytes(data).unwrap();
        assert_eq!(hash1, hash2);
    }
    
    #[tokio::test]
    async fn test_artifact_from_bytes() {
        let data = b"test artifact data".to_vec();
        let artifact = Artifact::from_bytes(data.clone(), "binary".to_string(), "test".to_string()).unwrap();
        
        assert_eq!(artifact.size(), data.len() as u64);
        assert_eq!(artifact.data(), data.as_slice());
    }
    
    #[tokio::test]
    async fn test_metadata_creation() {
        let hash = ArtifactHash::new("test_hash".to_string());
        let metadata = ArtifactMetadata::new(hash, 1024, "binary".to_string(), "test".to_string());
        
        assert_eq!(metadata.size, 1024);
        assert_eq!(metadata.artifact_type, "binary");
        assert!(metadata.compressed_size.is_none());
        assert!(metadata.timestamp > 0);
    }
    
    #[tokio::test]
    async fn test_compression_ratio() {
        let hash = ArtifactHash::new("test_hash".to_string());
        let metadata = ArtifactMetadata::new(hash, 1000, "binary".to_string(), "test".to_string())
            .with_compression(500, "zstd".to_string());
        
        assert_eq!(metadata.compression_ratio(), Some(0.5));
    }
}
