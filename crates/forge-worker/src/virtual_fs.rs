#![forbid(unsafe_code)]

//! Virtual File System for distributed workers
//! 
//! This module provides a virtual file system interface for distributed build workers,
//! allowing on-demand file access without copying entire source trees.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;

/// Virtual file system node
#[derive(Debug, Clone)]
pub enum VfsNode {
    File {
        content: Vec<u8>,
        metadata: FileMetadata,
    },
    Directory {
        children: HashMap<String, VfsNode>,
        metadata: FileMetadata,
    },
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: u64,
    pub is_executable: bool,
}

/// Virtual file system for distributed workers
pub struct VirtualFileSystem {
    root: Arc<RwLock<VfsNode>>,
    cache: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>,
    max_cache_size: usize,
}

impl VirtualFileSystem {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            root: Arc::new(RwLock::new(VfsNode::Directory {
                children: HashMap::new(),
                metadata: FileMetadata {
                    size: 0,
                    modified: 0,
                    is_executable: false,
                },
            })),
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size,
        }
    }

    /// Mount a local directory into the virtual file system
    pub fn mount_local(&self, local_path: &Path, vfs_path: &Path) -> Result<(), VfsError> {
        let local_path = std::fs::canonicalize(local_path)
            .map_err(|e| VfsError::IoError(e.to_string()))?;

        self.mount_recursive(&local_path, vfs_path)
    }

    fn mount_recursive(&self, local_path: &Path, vfs_path: &Path) -> Result<(), VfsError> {
        if local_path.is_file() {
            let content = std::fs::read(local_path)
                .map_err(|e| VfsError::IoError(e.to_string()))?;
            
            let modified = std::fs::metadata(local_path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            
            let metadata = FileMetadata {
                size: content.len() as u64,
                modified,
                is_executable: is_executable(local_path),
            };

            self.write_file(vfs_path, content, metadata)?;
        } else if local_path.is_dir() {
            let entries = std::fs::read_dir(local_path)
                .map_err(|e| VfsError::IoError(e.to_string()))?;

            for entry in entries {
                let entry = entry.map_err(|e| VfsError::IoError(e.to_string()))?;
                let entry_path = entry.path();
                let entry_name = entry.file_name()
                    .to_string_lossy()
                    .to_string();
                let child_vfs_path = vfs_path.join(&entry_name);
                
                self.mount_recursive(&entry_path, &child_vfs_path)?;
            }
        }

        Ok(())
    }

    /// Read a file from the virtual file system
    pub fn read_file(&self, path: &Path) -> Result<Vec<u8>, VfsError> {
        let cache = self.cache.read().unwrap();
        if let Some(cached) = cache.get(path) {
            return Ok(cached.clone());
        }
        drop(cache);

        let root = self.root.read().unwrap();
        let content = self.read_from_node(&root, path)?;
        drop(root);

        // Cache the result
        let mut cache = self.cache.write().unwrap();
        if cache.len() < self.max_cache_size {
            cache.insert(path.to_path_buf(), content.clone());
        }

        Ok(content)
    }

    fn read_from_node(&self, node: &VfsNode, path: &Path) -> Result<Vec<u8>, VfsError> {
        let components: Vec<&str> = path.iter()
            .filter_map(|c| c.to_str())
            .collect();

        if components.is_empty() {
            return Err(VfsError::InvalidPath("Empty path".to_string()));
        }

        self.traverse_node(node, &components, 0)
    }

    fn traverse_node(&self, node: &VfsNode, components: &[&str], index: usize) -> Result<Vec<u8>, VfsError> {
        match node {
            VfsNode::File { content, .. } => {
                if index == components.len() {
                    Ok(content.clone())
                } else {
                    Err(VfsError::InvalidPath("Path too long for file".to_string()))
                }
            }
            VfsNode::Directory { children, .. } => {
                if index >= components.len() {
                    return Err(VfsError::InvalidPath("Path too short for directory".to_string()));
                }

                let child_name = components[index];
                let child = children.get(child_name)
                    .ok_or_else(|| VfsError::NotFound(child_name.to_string()))?;

                self.traverse_node(child, components, index + 1)
            }
        }
    }

    /// Write a file to the virtual file system
    pub fn write_file(&self, path: &Path, content: Vec<u8>, metadata: FileMetadata) -> Result<(), VfsError> {
        let mut root = self.root.write().unwrap();
        self.write_to_node(&mut root, path, content, metadata)?;
        Ok(())
    }

    fn write_to_node(&self, node: &mut VfsNode, path: &Path, content: Vec<u8>, metadata: FileMetadata) -> Result<(), VfsError> {
        let components: Vec<&str> = path.iter()
            .filter_map(|c| c.to_str())
            .collect();

        if components.is_empty() {
            return Err(VfsError::InvalidPath("Empty path".to_string()));
        }

        self.traverse_and_write(node, &components, 0, content, metadata)
    }

    fn traverse_and_write(&self, node: &mut VfsNode, components: &[&str], index: usize, content: Vec<u8>, metadata: FileMetadata) -> Result<(), VfsError> {
        match node {
            VfsNode::File { .. } => {
                if index == components.len() {
                    // Replace existing file
                    *node = VfsNode::File { content, metadata };
                    Ok(())
                } else {
                    Err(VfsError::InvalidPath("Cannot traverse through file".to_string()))
                }
            }
            VfsNode::Directory { children, .. } => {
                if index >= components.len() {
                    return Err(VfsError::InvalidPath("Path too short".to_string()));
                }

                let child_name = components[index];
                
                if index == components.len() - 1 {
                    // This is the final component, create file
                    children.insert(child_name.to_string(), VfsNode::File { content, metadata });
                    Ok(())
                } else {
                    // Need to traverse further
                    let child = children.entry(child_name.to_string())
                        .or_insert_with(|| VfsNode::Directory {
                            children: HashMap::new(),
                            metadata: FileMetadata {
                                size: 0,
                                modified: 0,
                                is_executable: false,
                            },
                        });
                    
                    self.traverse_and_write(child, components, index + 1, content, metadata)
                }
            }
        }
    }

    /// Check if a file exists
    pub fn exists(&self, path: &Path) -> bool {
        let root = self.root.read().unwrap();
        self.check_exists(&root, path)
    }

    fn check_exists(&self, node: &VfsNode, path: &Path) -> bool {
        let components: Vec<&str> = path.iter()
            .filter_map(|c| c.to_str())
            .collect();

        if components.is_empty() {
            return true; // Root exists
        }

        self.traverse_exists(node, &components, 0)
    }

    fn traverse_exists(&self, node: &VfsNode, components: &[&str], index: usize) -> bool {
        match node {
            VfsNode::File { .. } => index == components.len(),
            VfsNode::Directory { children, .. } => {
                if index >= components.len() {
                    return false;
                }

                let child_name = components[index];
                if let Some(child) = children.get(child_name) {
                    self.traverse_exists(child, components, index + 1)
                } else {
                    false
                }
            }
        }
    }

    /// Get file metadata
    pub fn metadata(&self, path: &Path) -> Result<FileMetadata, VfsError> {
        let root = self.root.read().unwrap();
        self.get_metadata_from_node(&root, path)
    }

    fn get_metadata_from_node(&self, node: &VfsNode, path: &Path) -> Result<FileMetadata, VfsError> {
        let components: Vec<&str> = path.iter()
            .filter_map(|c| c.to_str())
            .collect();

        let (_, metadata) = self.traverse_metadata(node, &components, 0)?;
        Ok(metadata)
    }

    fn traverse_metadata(&self, node: &VfsNode, components: &[&str], index: usize) -> Result<(bool, FileMetadata), VfsError> {
        match node {
            VfsNode::File { metadata, .. } => {
                if index == components.len() {
                    Ok((true, metadata.clone()))
                } else {
                    Err(VfsError::InvalidPath("Path too long for file".to_string()))
                }
            }
            VfsNode::Directory { children, metadata, .. } => {
                if index >= components.len() {
                    return Ok((false, metadata.clone()));
                }

                let child_name = components[index];
                let child = children.get(child_name)
                    .ok_or_else(|| VfsError::NotFound(child_name.to_string()))?;

                self.traverse_metadata(child, components, index + 1)
            }
        }
    }

    /// List directory contents
    pub fn list_directory(&self, path: &Path) -> Result<Vec<String>, VfsError> {
        let root = self.root.read().unwrap();
        let components: Vec<&str> = path.iter()
            .filter_map(|c| c.to_str())
            .collect();

        let children = self.traverse_list(&root, &components, 0)?;
        Ok(children)
    }

    fn traverse_list(&self, node: &VfsNode, components: &[&str], index: usize) -> Result<Vec<String>, VfsError> {
        match node {
            VfsNode::File { .. } => {
                Err(VfsError::InvalidPath("Cannot list file".to_string()))
            }
            VfsNode::Directory { children, .. } => {
                if index >= components.len() {
                    Ok(children.keys().cloned().collect())
                } else {
                    let child_name = components[index];
                    let child = children.get(child_name)
                        .ok_or_else(|| VfsError::NotFound(child_name.to_string()))?;
                    self.traverse_list(child, components, index + 1)
                }
            }
        }
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        CacheStats {
            entries: cache.len(),
            total_size: cache.values().map(|v| v.len()).sum(),
            max_size: self.max_cache_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub total_size: usize,
    pub max_size: usize,
}

#[derive(Debug, Clone)]
pub enum VfsError {
    NotFound(String),
    InvalidPath(String),
    IoError(String),
}

impl std::fmt::Display for VfsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VfsError::NotFound(path) => write!(f, "Path not found: {}", path),
            VfsError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
            VfsError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for VfsError {}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    
    #[cfg(windows)]
    {
        // On Windows, check file extension
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| matches!(ext.to_lowercase().as_str(), "exe" | "bat" | "cmd" | "ps1"))
            .unwrap_or(false)
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_creation() {
        let vfs = VirtualFileSystem::new(100);
        // Test that VFS was created successfully by checking it can handle operations
        let test_path = Path::new("/test");
        let content = b"test".to_vec();
        let metadata = FileMetadata {
            size: 4,
            modified: 0,
            is_executable: false,
        };
        assert!(vfs.write_file(test_path, content, metadata).is_ok());
        assert!(vfs.exists(test_path));
    }

    #[test]
    fn test_file_operations() {
        let vfs = VirtualFileSystem::new(100);
        let path = Path::new("/test.txt");
        let content = b"test content".to_vec();
        let metadata = FileMetadata {
            size: content.len() as u64,
            modified: 0,
            is_executable: false,
        };

        vfs.write_file(path, content.clone(), metadata).unwrap();
        
        assert!(vfs.exists(path));
        let read_content = vfs.read_file(path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_directory_operations() {
        let vfs = VirtualFileSystem::new(100);
        let dir_path = Path::new("/test");
        let file_path = Path::new("/test/file.txt");
        
        let content = b"test".to_vec();
        let metadata = FileMetadata {
            size: content.len() as u64,
            modified: 0,
            is_executable: false,
        };

        vfs.write_file(file_path, content, metadata).unwrap();
        
        let children = vfs.list_directory(dir_path).unwrap();
        assert_eq!(children.len(), 1);
        assert!(children.contains(&"file.txt".to_string()));
    }

    #[test]
    fn test_cache_operations() {
        let vfs = VirtualFileSystem::new(10);
        let path = Path::new("/cached.txt");
        let content = b"cached content".to_vec();
        let metadata = FileMetadata {
            size: content.len() as u64,
            modified: 0,
            is_executable: false,
        };

        vfs.write_file(path, content.clone(), metadata).unwrap();
        
        // First read caches the file
        vfs.read_file(path).unwrap();
        
        let stats = vfs.cache_stats();
        assert_eq!(stats.entries, 1);
        assert!(stats.total_size > 0);
    }
}