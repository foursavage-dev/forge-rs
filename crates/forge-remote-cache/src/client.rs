#![forbid(unsafe_code)]

use std::fmt::Debug;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

use base64::Engine;

use crate::protocol::{CacheRequest, CacheResponse};
use crate::{RemoteCacheClient, RemoteCacheError};

#[derive(Debug, Clone)]
pub struct TcpRemoteCacheClient {
    pub server_addr: String,
    pub auth_token: Option<String>,
    pub timeout: Duration,
}

impl TcpRemoteCacheClient {
    pub fn new(server_addr: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            server_addr: server_addr.into(),
            auth_token,
            timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn send_request(&self, req: CacheRequest) -> Result<CacheResponse, RemoteCacheError> {
        let mut stream = TcpStream::connect(&self.server_addr).map_err(|e| {
            RemoteCacheError::Network(format!("cannot connect to {}: {e}", self.server_addr))
        })?;

        let _ = stream.set_read_timeout(Some(self.timeout));
        let _ = stream.set_write_timeout(Some(self.timeout));

        let req_json = serde_json::to_vec(&req)
            .map_err(|e| RemoteCacheError::Protocol(format!("cannot serialize request: {e}")))?;

        stream
            .write_all(&req_json)
            .map_err(|e| RemoteCacheError::Network(format!("failed to send request: {e}")))?;
        stream
            .write_all(b"\n")
            .map_err(|e| RemoteCacheError::Network(format!("failed to send newline: {e}")))?;
        stream
            .flush()
            .map_err(|e| RemoteCacheError::Network(format!("failed to flush stream: {e}")))?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        if reader
            .read_line(&mut line)
            .map_err(|e| RemoteCacheError::Network(format!("failed to read response: {e}")))?
            == 0
        {
            return Err(RemoteCacheError::Protocol(
                "empty response from server".to_string(),
            ));
        }

        let resp: CacheResponse = serde_json::from_str(line.trim())
            .map_err(|e| RemoteCacheError::Protocol(format!("cannot parse response: {e}")))?;

        Ok(resp)
    }

    pub fn ping(&self) -> Result<bool, RemoteCacheError> {
        let req = CacheRequest::Ping {
            auth_token: self.auth_token.clone(),
        };
        match self.send_request(req)? {
            CacheResponse::Pong { status, error, .. } => {
                if let Some(err) = error {
                    Err(RemoteCacheError::Protocol(err))
                } else {
                    Ok(status == "ok")
                }
            }
            CacheResponse::Error { message } => Err(RemoteCacheError::Protocol(message)),
            _ => Err(RemoteCacheError::Protocol(
                "unexpected response to ping".to_string(),
            )),
        }
    }

    pub fn get_artifact(&self, hash: &str) -> Result<Option<Vec<u8>>, RemoteCacheError> {
        let req = CacheRequest::GetArtifact {
            hash: hash.to_string(),
            auth_token: self.auth_token.clone(),
        };
        match self.send_request(req)? {
            CacheResponse::Artifact {
                found,
                data_base64,
                error,
                ..
            } => {
                if let Some(err) = error {
                    return Err(RemoteCacheError::Protocol(err));
                }
                if !found {
                    return Ok(None);
                }
                if let Some(encoded) = data_base64 {
                    let bytes = base64::engine::general_purpose::STANDARD
                        .decode(&encoded)
                        .map_err(|e| {
                            RemoteCacheError::Protocol(format!("invalid base64 artifact: {e}"))
                        })?;
                    Ok(Some(bytes))
                } else {
                    Ok(None)
                }
            }
            CacheResponse::Error { message } => Err(RemoteCacheError::Protocol(message)),
            _ => Err(RemoteCacheError::Protocol(
                "unexpected response to get_artifact".to_string(),
            )),
        }
    }

    pub fn put_artifact(&self, hash: &str, data: &[u8]) -> Result<(), RemoteCacheError> {
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        let req = CacheRequest::PutArtifact {
            hash: hash.to_string(),
            data_base64: encoded,
            auth_token: self.auth_token.clone(),
        };

        match self.send_request(req)? {
            CacheResponse::PutArtifactResult { success, error, .. } => {
                if let Some(err) = error {
                    Err(RemoteCacheError::Protocol(err))
                } else if success {
                    Ok(())
                } else {
                    Err(RemoteCacheError::Protocol(
                        "failed to put artifact".to_string(),
                    ))
                }
            }
            CacheResponse::Error { message } => Err(RemoteCacheError::Protocol(message)),
            _ => Err(RemoteCacheError::Protocol(
                "unexpected response to put_artifact".to_string(),
            )),
        }
    }
}

impl RemoteCacheClient for TcpRemoteCacheClient {
    fn get_fingerprint(&self, key: &str) -> Result<Option<String>, RemoteCacheError> {
        let req = CacheRequest::GetFingerprint {
            key: key.to_string(),
            auth_token: self.auth_token.clone(),
        };

        match self.send_request(req)? {
            CacheResponse::Fingerprint {
                found,
                fingerprint,
                error,
                ..
            } => {
                if let Some(err) = error {
                    Err(RemoteCacheError::Protocol(err))
                } else if found {
                    Ok(fingerprint)
                } else {
                    Ok(None)
                }
            }
            CacheResponse::Error { message } => Err(RemoteCacheError::Protocol(message)),
            _ => Err(RemoteCacheError::Protocol(
                "unexpected response to get_fingerprint".to_string(),
            )),
        }
    }

    fn put_fingerprint(&self, key: &str, fingerprint: &str) -> Result<(), RemoteCacheError> {
        let req = CacheRequest::PutFingerprint {
            key: key.to_string(),
            fingerprint: fingerprint.to_string(),
            auth_token: self.auth_token.clone(),
        };

        match self.send_request(req)? {
            CacheResponse::PutFingerprintResult { success, error, .. } => {
                if let Some(err) = error {
                    Err(RemoteCacheError::Protocol(err))
                } else if success {
                    Ok(())
                } else {
                    Err(RemoteCacheError::Protocol(
                        "failed to put fingerprint".to_string(),
                    ))
                }
            }
            CacheResponse::Error { message } => Err(RemoteCacheError::Protocol(message)),
            _ => Err(RemoteCacheError::Protocol(
                "unexpected response to put_fingerprint".to_string(),
            )),
        }
    }
}
