#![forbid(unsafe_code)]

pub mod client;
pub mod cluster;
pub mod protocol;
pub mod server;
pub mod virtual_fs;

pub use client::RemoteWorkerClient;
pub use cluster::ClusterExecutor;
pub use protocol::{
    RemoteTaskRequest, RemoteTaskResponse, WorkerHealthInfo, WorkerPingRequest, WorkerPingResponse,
    VfsFileRequest, VfsFileResponse, VfsFileMetadata,
};
pub use server::WorkerServer;
pub use virtual_fs::{VirtualFileSystem, VfsNode, FileMetadata, VfsError, CacheStats};
