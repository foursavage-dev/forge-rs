#![forbid(unsafe_code)]

//! Async file I/O operations for cache
//!
//! This module provides async file operations to improve I/O performance
//! and reduce blocking during cache operations.
//!
//! Performance optimizations:
//! - Async file operations with Tokio
//! - Batched reads/writes for better throughput
//! - Parallel file operations where possible

use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Semaphore;

#[derive(Debug, Error)]
pub enum AsyncIoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

/// Async file writer with batching support
pub struct AsyncFileWriter {
    max_parallel: usize,
    semaphore: Arc<Semaphore>,
}

impl AsyncFileWriter {
    pub fn new(max_parallel: usize) -> Self {
        Self {
            max_parallel: max_parallel.max(1),
            semaphore: Arc::new(Semaphore::new(max_parallel.max(1))),
        }
    }

    /// Write data to a file atomically (write to temp, then rename)
    pub async fn write_atomic(&self, path: PathBuf, data: Vec<u8>) -> Result<(), AsyncIoError> {
        let _permit = self.semaphore.acquire().await.unwrap();

        let tmp_path = crate::unique_tmp_path(&path);

        // Write to temp file
        let mut file = fs::File::create(&tmp_path).await?;
        file.write_all(&data).await?;
        file.flush().await?;
        file.sync_all().await?;
        drop(file);

        // Atomic rename
        fs::rename(&tmp_path, &path).await?;

        Ok(())
    }

    /// Batch write multiple files in parallel
    pub async fn write_batch(
        &self,
        operations: Vec<(PathBuf, Vec<u8>)>,
    ) -> Result<Vec<Result<(), AsyncIoError>>, AsyncIoError> {
        let mut tasks = Vec::new();

        for (path, data) in operations {
            let writer = self.clone();
            let task = tokio::spawn(async move { writer.write_atomic(path, data).await });
            tasks.push(task);
        }

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(e.into())),
            }
        }

        Ok(results)
    }
}

impl Clone for AsyncFileWriter {
    fn clone(&self) -> Self {
        Self {
            max_parallel: self.max_parallel,
            semaphore: self.semaphore.clone(),
        }
    }
}

/// Async file reader with prefetch support
pub struct AsyncFileReader {
    max_parallel: usize,
    semaphore: Arc<Semaphore>,
}

impl AsyncFileReader {
    pub fn new(max_parallel: usize) -> Self {
        Self {
            max_parallel: max_parallel.max(1),
            semaphore: Arc::new(Semaphore::new(max_parallel.max(1))),
        }
    }

    /// Read entire file asynchronously
    pub async fn read_file(&self, path: &Path) -> Result<Vec<u8>, AsyncIoError> {
        let _permit = self.semaphore.acquire().await.unwrap();

        let mut file = fs::File::open(path).await?;
        let metadata = file.metadata().await?;
        let mut buffer = Vec::with_capacity(metadata.len() as usize);
        file.read_to_end(&mut buffer).await?;

        Ok(buffer)
    }

    /// Batch read multiple files in parallel
    pub async fn read_batch(
        &self,
        paths: Vec<PathBuf>,
    ) -> Result<Vec<Result<Vec<u8>, AsyncIoError>>, AsyncIoError> {
        let mut tasks = Vec::new();

        for path in paths {
            let reader = self.clone();
            let task = tokio::spawn(async move { reader.read_file(&path).await });
            tasks.push(task);
        }

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(e.into())),
            }
        }

        Ok(results)
    }
}

impl Clone for AsyncFileReader {
    fn clone(&self) -> Self {
        Self {
            max_parallel: self.max_parallel,
            semaphore: self.semaphore.clone(),
        }
    }
}

/// Async cache operations helper
pub struct AsyncCache {
    writer: AsyncFileWriter,
    reader: AsyncFileReader,
}

impl AsyncCache {
    pub fn new(max_parallel: usize) -> Self {
        Self {
            writer: AsyncFileWriter::new(max_parallel),
            reader: AsyncFileReader::new(max_parallel),
        }
    }

    /// Write fingerprint record asynchronously
    pub async fn write_fingerprint(
        &self,
        path: PathBuf,
        fingerprint: &str,
        artifact_hash: Option<String>,
    ) -> Result<(), AsyncIoError> {
        let record = serde_json::json!({
            "fingerprint": fingerprint,
            "stored_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "artifact_hash": artifact_hash,
        });

        let data = serde_json::to_vec(&record)?;
        self.writer.write_atomic(path, data).await
    }

    /// Read fingerprint record asynchronously
    pub async fn read_fingerprint(
        &self,
        path: &Path,
    ) -> Result<Option<(String, Option<String>)>, AsyncIoError> {
        match self.reader.read_file(path).await {
            Ok(data) => {
                let record: serde_json::Value = serde_json::from_slice(&data)?;
                let fingerprint = record["fingerprint"].as_str().unwrap_or("").to_string();
                let artifact_hash = record["artifact_hash"].as_str().map(|s| s.to_string());
                Ok(Some((fingerprint, artifact_hash)))
            }
            Err(AsyncIoError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Batch read multiple fingerprint records
    pub async fn read_fingerprints_batch(
        &self,
        paths: Vec<PathBuf>,
    ) -> Result<Vec<Option<(String, Option<String>)>>, AsyncIoError> {
        let results = self.reader.read_batch(paths).await?;

        let mut fingerprints = Vec::new();
        for result in results {
            match result {
                Ok(data) => {
                    let record: serde_json::Value = serde_json::from_slice(&data)?;
                    let fingerprint = record["fingerprint"].as_str().unwrap_or("").to_string();
                    let artifact_hash = record["artifact_hash"].as_str().map(|s| s.to_string());
                    fingerprints.push(Some((fingerprint, artifact_hash)));
                }
                Err(AsyncIoError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                    fingerprints.push(None);
                }
                Err(e) => return Err(e),
            }
        }

        Ok(fingerprints)
    }
}

impl Default for AsyncCache {
    fn default() -> Self {
        Self::new(8) // Default to 8 parallel operations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::runtime::Runtime;

    #[test]
    fn test_async_file_writer() {
        let rt = Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        rt.block_on(async {
            let writer = AsyncFileWriter::new(4);
            writer
                .write_atomic(file_path.clone(), b"test data".to_vec())
                .await
                .unwrap();

            let content = fs::read_to_string(&file_path).await.unwrap();
            assert_eq!(content, "test data");
        });
    }

    #[test]
    fn test_async_file_reader() {
        let rt = Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        rt.block_on(async {
            fs::write(&file_path, b"test data").await.unwrap();

            let reader = AsyncFileReader::new(4);
            let content = reader.read_file(&file_path).await.unwrap();
            assert_eq!(content, b"test data");
        });
    }

    #[test]
    fn test_async_cache_operations() {
        let rt = Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("fingerprint.json");

        rt.block_on(async {
            let cache = AsyncCache::new(4);
            cache
                .write_fingerprint(file_path.clone(), "test_fp", Some("hash123".to_string()))
                .await
                .unwrap();

            let result = cache.read_fingerprint(&file_path).await.unwrap();
            assert!(result.is_some());
            let (fp, hash) = result.unwrap();
            assert_eq!(fp, "test_fp");
            assert_eq!(hash, Some("hash123".to_string()));
        });
    }

    #[test]
    fn test_batch_operations() {
        let rt = Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();

        rt.block_on(async {
            let writer = AsyncFileWriter::new(4);

            let operations = (0..10)
                .map(|i| {
                    (
                        temp_dir.path().join(format!("file_{}.txt", i)),
                        format!("content_{}", i).into_bytes(),
                    )
                })
                .collect();

            writer.write_batch(operations).await.unwrap();

            let reader = AsyncFileReader::new(4);
            let paths = (0..10)
                .map(|i| temp_dir.path().join(format!("file_{}.txt", i)))
                .collect();
            let results = reader.read_batch(paths).await.unwrap();

            assert_eq!(results.len(), 10);
            for (i, result) in results.into_iter().enumerate() {
                let content = result.unwrap();
                assert_eq!(
                    String::from_utf8(content).unwrap(),
                    format!("content_{}", i)
                );
            }
        });
    }
}
