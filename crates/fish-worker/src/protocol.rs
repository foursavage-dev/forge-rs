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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_usage_pct: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_used_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_total_bytes: Option<u64>,
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

pub const BINARY_MAGIC: &[u8; 4] = b"FORG";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum FrameType {
    TaskRequest = 1,
    TaskResponse = 2,
    VfsRequest = 3,
    VfsResponse = 4,
    Ping = 5,
    Pong = 6,
    RawPayload = 7,
}

impl TryFrom<u16> for FrameType {
    type Error = std::io::Error;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        match val {
            1 => Ok(FrameType::TaskRequest),
            2 => Ok(FrameType::TaskResponse),
            3 => Ok(FrameType::VfsRequest),
            4 => Ok(FrameType::VfsResponse),
            5 => Ok(FrameType::Ping),
            6 => Ok(FrameType::Pong),
            7 => Ok(FrameType::RawPayload),
            other => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown frame type {other}"),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryFrame {
    pub frame_type: FrameType,
    pub flags: u16,
    pub payload: Vec<u8>,
}

impl BinaryFrame {
    pub fn new(frame_type: FrameType, payload: Vec<u8>) -> Self {
        Self {
            frame_type,
            flags: 0,
            payload,
        }
    }

    pub fn with_flags(frame_type: FrameType, flags: u16, payload: Vec<u8>) -> Self {
        Self {
            frame_type,
            flags,
            payload,
        }
    }

    pub fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(BINARY_MAGIC)?;
        writer.write_all(&(self.frame_type as u16).to_be_bytes())?;
        writer.write_all(&self.flags.to_be_bytes())?;
        writer.write_all(&(self.payload.len() as u64).to_be_bytes())?;
        writer.write_all(&self.payload)?;
        writer.flush()?;
        Ok(())
    }

    pub fn decode<R: std::io::Read>(reader: &mut R) -> std::io::Result<Option<Self>> {
        let mut magic = [0u8; 4];
        match reader.read_exact(&mut magic) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        }

        if &magic != BINARY_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid binary magic header",
            ));
        }

        let mut type_buf = [0u8; 2];
        reader.read_exact(&mut type_buf)?;
        let frame_type = FrameType::try_from(u16::from_be_bytes(type_buf))?;

        let mut flags_buf = [0u8; 2];
        reader.read_exact(&mut flags_buf)?;
        let flags = u16::from_be_bytes(flags_buf);

        let mut len_buf = [0u8; 8];
        reader.read_exact(&mut len_buf)?;
        let payload_len = u64::from_be_bytes(len_buf) as usize;

        let mut payload = vec![0u8; payload_len];
        reader.read_exact(&mut payload)?;

        Ok(Some(Self {
            frame_type,
            flags,
            payload,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_frame_roundtrip() {
        let frame = BinaryFrame::with_flags(
            FrameType::TaskRequest,
            0x0001,
            b"test binary payload data 12345".to_vec(),
        );
        let mut buffer = Vec::new();
        frame.encode(&mut buffer).unwrap();

        let mut cursor = std::io::Cursor::new(buffer);
        let decoded = BinaryFrame::decode(&mut cursor).unwrap().unwrap();

        assert_eq!(frame, decoded);
    }

    #[test]
    fn test_binary_frame_invalid_magic() {
        let buffer = b"BADM\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        let mut cursor = std::io::Cursor::new(buffer);
        assert!(BinaryFrame::decode(&mut cursor).is_err());
    }
}
