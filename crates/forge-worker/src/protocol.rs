#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteTaskRequest {
    pub task_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    pub auth_token: Option<String>,
    pub timeout_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceContext>,
}

/// A tar.zst snapshot of the task's working tree, base64-encoded on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceContext {
    /// Absolute path of the packed tree on the sending machine. The worker
    /// uses it to re-resolve `cwd` inside the extracted snapshot.
    pub root: String,
    /// tar.zst payload, base64-encoded.
    pub data_base64: String,
    pub format: String,
    /// Enable VFS mode for on-demand file streaming
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_vfs: Option<bool>,
    /// VFS mount point for streaming
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vfs_mount: Option<String>,
}

/// Request to stream a specific file from VFS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VfsFileRequest {
    pub file_path: String,
    pub auth_token: Option<String>,
}

/// Response with file content from VFS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VfsFileResponse {
    pub success: bool,
    pub content_base64: Option<String>,
    pub error: Option<String>,
    pub metadata: Option<VfsFileMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VfsFileMetadata {
    pub size: u64,
    pub modified: u64,
    pub is_executable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteTaskResponse {
    pub task_id: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHealthInfo {
    pub worker_name: String,
    pub active_jobs: usize,
    pub max_concurrency: usize,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerPingRequest {
    pub auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPingResponse {
    pub status: String,
    pub health: WorkerHealthInfo,
    pub error: Option<String>,
}
