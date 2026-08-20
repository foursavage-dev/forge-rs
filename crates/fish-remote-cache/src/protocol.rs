#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheRequest {
    GetFingerprint {
        key: String,
        auth_token: Option<String>,
    },
    PutFingerprint {
        key: String,
        fingerprint: String,
        auth_token: Option<String>,
    },
    GetArtifact {
        hash: String,
        auth_token: Option<String>,
    },
    PutArtifact {
        hash: String,
        data_base64: String,
        auth_token: Option<String>,
    },
    Ping {
        auth_token: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheResponse {
    Fingerprint {
        key: String,
        found: bool,
        fingerprint: Option<String>,
        error: Option<String>,
    },
    PutFingerprintResult {
        key: String,
        success: bool,
        error: Option<String>,
    },
    Artifact {
        hash: String,
        found: bool,
        data_base64: Option<String>,
        error: Option<String>,
    },
    PutArtifactResult {
        hash: String,
        success: bool,
        error: Option<String>,
    },
    Pong {
        status: String,
        version: String,
        stored_entries: usize,
        error: Option<String>,
    },
    Error {
        message: String,
    },
}
