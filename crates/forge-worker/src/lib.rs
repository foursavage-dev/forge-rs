#![forbid(unsafe_code)]

pub mod client;
pub mod cluster;
pub mod protocol;
pub mod server;
pub mod virtual_fs;

pub use client::RemoteWorkerClient;
pub use cluster::ClusterExecutor;
pub use protocol::{
    RemoteTaskRequest, RemoteTaskResponse, VfsFileMetadata, VfsFileRequest, VfsFileResponse,
    WorkerHealthInfo, WorkerPingRequest, WorkerPingResponse,
};
pub use server::WorkerServer;
pub use virtual_fs::{CacheStats, FileMetadata, VfsError, VfsNode, VirtualFileSystem};
