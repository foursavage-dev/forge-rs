#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use base64::Engine;

use crate::artifact::blob_hash;
use crate::protocol::{CacheRequest, CacheResponse};

pub struct RemoteCacheServer {
    addr: String,
    auth_token: Option<String>,
    storage_dir: Option<PathBuf>,
    fingerprints: Arc<RwLock<HashMap<String, String>>>,
    artifacts: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    artifact_index: Arc<RwLock<HashMap<String, String>>>,
    running: Arc<AtomicBool>,
}

impl RemoteCacheServer {
    pub fn new(
        addr: impl Into<String>,
        auth_token: Option<String>,
        storage_dir: Option<PathBuf>,
    ) -> Self {
        let storage = storage_dir.clone();
        let fingerprints = Arc::new(RwLock::new(HashMap::new()));
        let artifacts = Arc::new(RwLock::new(HashMap::new()));
        let artifact_index = Arc::new(RwLock::new(HashMap::new()));

        if let Some(dir) = &storage {
            let fp_dir = dir.join("fingerprints");
            let art_dir = dir.join("artifacts");
            let _ = fs::create_dir_all(&fp_dir);
            let _ = fs::create_dir_all(&art_dir);
            let _ = fs::create_dir_all(art_dir.join("objects"));

            if let Ok(entries) = fs::read_dir(&fp_dir) {
                let mut guard = fingerprints.write().unwrap();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && let Some(name) = path.file_stem().and_then(|s| s.to_str())
                        && let Ok(content) = fs::read_to_string(&path)
                    {
                        guard.insert(name.to_string(), content.trim().to_string());
                    }
                }
            }

            let index_path = art_dir.join("index.json");
            if let Ok(content) = fs::read_to_string(index_path)
                && let Ok(index) = serde_json::from_str::<HashMap<String, String>>(&content)
            {
                let mut guard = artifact_index.write().unwrap();
                guard.extend(index);
            }
        }

        Self {
            addr: addr.into(),
            auth_token,
            storage_dir,
            fingerprints,
            artifacts,
            artifact_index,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn start_background(&self) -> std::io::Result<JoinHandle<()>> {
        let listener = TcpListener::bind(&self.addr)?;
        let local_addr = listener.local_addr()?.to_string();
        let auth_token = self.auth_token.clone();
        let storage_dir = self.storage_dir.clone();
        let fingerprints = Arc::clone(&self.fingerprints);
        let artifacts = Arc::clone(&self.artifacts);
        let artifact_index = Arc::clone(&self.artifact_index);
        let running = Arc::clone(&self.running);

        running.store(true, Ordering::SeqCst);
        let _ = listener.set_nonblocking(true);

        let handle = thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let auth = auth_token.clone();
                        let storage = storage_dir.clone();
                        let fp = Arc::clone(&fingerprints);
                        let art = Arc::clone(&artifacts);
                        let idx = Arc::clone(&artifact_index);
                        thread::spawn(move || {
                            let _ = Self::handle_connection(
                                stream,
                                &auth,
                                storage.as_deref(),
                                fp,
                                art,
                                idx,
                            );
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        let _ = local_addr;
        Ok(handle)
    }

    pub fn run_blocking(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.addr)?;
        self.running.store(true, Ordering::SeqCst);

        while self.running.load(Ordering::SeqCst) {
            let (stream, _) = listener.accept()?;
            let auth = self.auth_token.clone();
            let storage = self.storage_dir.clone();
            let fp = Arc::clone(&self.fingerprints);
            let art = Arc::clone(&self.artifacts);
            let idx = Arc::clone(&self.artifact_index);
            thread::spawn(move || {
                let _ = Self::handle_connection(stream, &auth, storage.as_deref(), fp, art, idx);
            });
        }
        Ok(())
    }

    pub fn handle_connection(
        mut stream: TcpStream,
        expected_token: &Option<String>,
        storage_dir: Option<&Path>,
        fingerprints: Arc<RwLock<HashMap<String, String>>>,
        artifacts: Arc<RwLock<HashMap<String, Vec<u8>>>>,
        artifact_index: Arc<RwLock<HashMap<String, String>>>,
    ) -> Result<(), anyhow::Error> {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));

        let reader_stream = stream.try_clone()?;
        let mut reader = BufReader::new(reader_stream);
        let mut line = String::new();

        while reader.read_line(&mut line)? > 0 {
            if line.len() > 64 * 1024 * 1024 {
                return Err(anyhow::anyhow!("request line exceeds maximum size limit"));
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                line.clear();
                continue;
            }

            let request: Result<CacheRequest, _> = serde_json::from_str(trimmed);
            let response = match request {
                Ok(req) => Self::process_request(
                    req,
                    expected_token,
                    storage_dir,
                    &fingerprints,
                    &artifacts,
                    &artifact_index,
                ),
                Err(err) => CacheResponse::Error {
                    message: format!("invalid request format: {err}"),
                },
            };

            let response_bytes = serde_json::to_vec(&response)?;
            stream.write_all(&response_bytes)?;
            stream.write_all(b"\n")?;
            stream.flush()?;
            line.clear();
        }

        Ok(())
    }

    fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut diff = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            diff |= x ^ y;
        }
        diff == 0
    }

    fn safe_key_identifier(key: &str) -> Option<String> {
        if key.is_empty() || key.len() > 256 {
            return None;
        }
        if key.contains("..") || key.contains('/') || key.contains('\\') || key.contains('\0') {
            return None;
        }
        if key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            Some(key.to_string())
        } else {
            None
        }
    }

    fn check_auth(expected_token: &Option<String>, token: &Option<String>) -> bool {
        match expected_token {
            Some(expected) => match token {
                Some(tok) => Self::constant_time_eq(expected.as_bytes(), tok.as_bytes()),
                None => false,
            },
            None => true,
        }
    }

    fn process_request(
        req: CacheRequest,
        expected_token: &Option<String>,
        storage_dir: Option<&Path>,
        fingerprints: &Arc<RwLock<HashMap<String, String>>>,
        artifacts: &Arc<RwLock<HashMap<String, Vec<u8>>>>,
        artifact_index: &Arc<RwLock<HashMap<String, String>>>,
    ) -> CacheResponse {
        match req {
            CacheRequest::Ping { auth_token } => {
                if !Self::check_auth(expected_token, &auth_token) {
                    return CacheResponse::Pong {
                        status: "unauthorized".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        stored_entries: 0,
                        error: Some("invalid authentication token".to_string()),
                    };
                }
                let guard = fingerprints.read().unwrap();
                CacheResponse::Pong {
                    status: "ok".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    stored_entries: guard.len(),
                    error: None,
                }
            }
            CacheRequest::GetFingerprint { key, auth_token } => {
                if !Self::check_auth(expected_token, &auth_token) {
                    return CacheResponse::Fingerprint {
                        key,
                        found: false,
                        fingerprint: None,
                        error: Some("unauthorized".to_string()),
                    };
                }
                let guard = fingerprints.read().unwrap();
                if let Some(fp) = guard.get(&key) {
                    CacheResponse::Fingerprint {
                        key,
                        found: true,
                        fingerprint: Some(fp.clone()),
                        error: None,
                    }
                } else {
                    CacheResponse::Fingerprint {
                        key,
                        found: false,
                        fingerprint: None,
                        error: None,
                    }
                }
            }
            CacheRequest::PutFingerprint {
                key,
                fingerprint,
                auth_token,
            } => {
                if !Self::check_auth(expected_token, &auth_token) {
                    return CacheResponse::PutFingerprintResult {
                        key,
                        success: false,
                        error: Some("unauthorized".to_string()),
                    };
                }
                {
                    let mut guard = fingerprints.write().unwrap();
                    guard.insert(key.clone(), fingerprint.clone());
                }

                if let Some(dir) = storage_dir
                    && let Some(safe_k) = Self::safe_key_identifier(&key)
                {
                    let fp_dir = dir.join("fingerprints");
                    let _ = fs::create_dir_all(&fp_dir);
                    let target_path = fp_dir.join(format!("{safe_k}.json"));
                    let tmp_path = fp_dir.join(format!("{safe_k}.tmp"));
                    if fs::write(&tmp_path, &fingerprint).is_ok() {
                        let _ = fs::rename(&tmp_path, &target_path);
                    }
                }

                CacheResponse::PutFingerprintResult {
                    key,
                    success: true,
                    error: None,
                }
            }
            CacheRequest::GetArtifact { hash, auth_token } => {
                if !Self::check_auth(expected_token, &auth_token) {
                    return CacheResponse::Artifact {
                        hash,
                        found: false,
                        data_base64: None,
                        error: Some("unauthorized".to_string()),
                    };
                }

                let in_memory = {
                    let guard = artifacts.read().unwrap();
                    guard.get(&hash).cloned()
                };

                let data_opt = if in_memory.is_some() {
                    in_memory
                } else if let Some(dir) = storage_dir {
                    let content_hash = {
                        let guard = artifact_index.read().unwrap();
                        guard.get(&hash).cloned()
                    };
                    content_hash.and_then(|h| {
                        Self::safe_key_identifier(&h).and_then(|safe_h| {
                            fs::read(dir.join("artifacts").join("objects").join(safe_h)).ok()
                        })
                    })
                } else {
                    None
                };

                if let Some(data) = data_opt {
                    let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
                    CacheResponse::Artifact {
                        hash,
                        found: true,
                        data_base64: Some(encoded),
                        error: None,
                    }
                } else {
                    CacheResponse::Artifact {
                        hash,
                        found: false,
                        data_base64: None,
                        error: None,
                    }
                }
            }
            CacheRequest::PutArtifact {
                hash,
                data_base64,
                auth_token,
            } => {
                if !Self::check_auth(expected_token, &auth_token) {
                    return CacheResponse::PutArtifactResult {
                        hash,
                        success: false,
                        error: Some("unauthorized".to_string()),
                    };
                }

                let bytes = match base64::engine::general_purpose::STANDARD.decode(&data_base64) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        return CacheResponse::PutArtifactResult {
                            hash,
                            success: false,
                            error: Some(format!("invalid base64 payload: {err}")),
                        };
                    }
                };

                let content_hash = blob_hash(&bytes);

                {
                    let mut guard = artifacts.write().unwrap();
                    guard.insert(hash.clone(), bytes.clone());
                }

                if let Some(dir) = storage_dir
                    && let Some(safe_h) = Self::safe_key_identifier(&content_hash)
                {
                    let art_dir = dir.join("artifacts");
                    let _ = fs::create_dir_all(art_dir.join("objects"));
                    let target_path = art_dir.join("objects").join(&safe_h);
                    let tmp_path = art_dir.join("objects").join(format!("{safe_h}.tmp"));
                    if fs::write(&tmp_path, &bytes).is_ok() {
                        let _ = fs::rename(&tmp_path, &target_path);
                    }
                    {
                        let mut guard = artifact_index.write().unwrap();
                        guard.insert(hash.clone(), safe_h);
                    }
                    let index_path = art_dir.join("index.json");
                    let snapshot = {
                        let guard = artifact_index.read().unwrap();
                        serde_json::to_vec(&*guard).unwrap_or_default()
                    };
                    let tmp_index = art_dir.join("index.json.tmp");
                    if fs::write(&tmp_index, snapshot).is_ok() {
                        let _ = fs::rename(&tmp_index, index_path);
                    }
                }

                CacheResponse::PutArtifactResult {
                    hash,
                    success: true,
                    error: None,
                }
            }
        }
    }
}
