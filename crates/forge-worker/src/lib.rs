#![forbid(unsafe_code)]

pub mod client;
pub mod protocol;
pub mod server;

pub use client::RemoteWorkerClient;
pub use protocol::{RemoteTaskRequest, RemoteTaskResponse, WorkerHealthInfo};
pub use server::WorkerServer;
