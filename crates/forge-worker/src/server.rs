#![forbid(unsafe_code)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::protocol::{
    RemoteTaskRequest, RemoteTaskResponse, SourceContext, VfsFileRequest, VfsFileResponse,
    WorkerHealthInfo, WorkerPingRequest, WorkerPingResponse,
};
use crate::virtual_fs::VirtualFileSystem;
use base64::Engine;
use forge_executor::{CommandSpec, ProcessExecutor, Task, TaskExecutor};
use forge_remote_cache::artifact::unpack_artifacts;

pub struct WorkerServer {
    addr: String,
    auth_token: Option<String>,
    worker_name: String,
    max_concurrency: usize,
    active_jobs: Arc<AtomicUsize>,
    start_time: Instant,
    running: Arc<AtomicBool>,
    vfs: Arc<VirtualFileSystem>,
}

impl WorkerServer {
    pub fn new(addr: impl Into<String>, auth_token: Option<String>) -> Self {
        Self::with_options(addr, auth_token, "forge-worker-node", 8)
    }

    pub fn with_options(
        addr: impl Into<String>,
        auth_token: Option<String>,
        worker_name: impl Into<String>,
        max_concurrency: usize,
    ) -> Self {
        Self {
            addr: addr.into(),
            auth_token,
            worker_name: worker_name.into(),
            max_concurrency: max_concurrency.max(1),
            active_jobs: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            running: Arc::new(AtomicBool::new(false)),
            vfs: Arc::new(VirtualFileSystem::new(1024 * 1024 * 100)), // 100MB cache
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

    pub fn vfs(&self) -> Arc<VirtualFileSystem> {
        Arc::clone(&self.vfs)
    }

    pub fn start_background(&self) -> std::io::Result<JoinHandle<()>> {
        let listener = TcpListener::bind(&self.addr)?;
        let auth_token = self.auth_token.clone();
        let worker_name = self.worker_name.clone();
        let max_concurrency = self.max_concurrency;
        let active_jobs = Arc::clone(&self.active_jobs);
        let start_time = self.start_time;
        let running = Arc::clone(&self.running);
        let vfs = Arc::clone(&self.vfs);

        running.store(true, Ordering::SeqCst);
        let _ = listener.set_nonblocking(true);

        let handle = thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let auth = auth_token.clone();
                        let name = worker_name.clone();
                        let jobs = Arc::clone(&active_jobs);
                        let vfs_clone = Arc::clone(&vfs);
                        thread::spawn(move || {
                            let _ = Self::handle_connection(
                                &mut stream,
                                &auth,
                                &name,
                                max_concurrency,
                                &jobs,
                                start_time,
                                vfs_clone,
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

        Ok(handle)
    }

    pub fn run_blocking(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.addr)?;
        self.running.store(true, Ordering::SeqCst);
        let vfs = Arc::clone(&self.vfs);

        while self.running.load(Ordering::SeqCst) {
            let (mut stream, _) = listener.accept()?;
            let auth = self.auth_token.clone();
            let name = self.worker_name.clone();
            let max_concurrency = self.max_concurrency;
            let jobs = Arc::clone(&self.active_jobs);
            let start_time = self.start_time;
            let vfs_clone = Arc::clone(&vfs);
            thread::spawn(move || {
                let _ = Self::handle_connection(
                    &mut stream,
                    &auth,
                    &name,
                    max_concurrency,
                    &jobs,
                    start_time,
                    vfs_clone,
                );
            });
        }
        Ok(())
    }

    pub fn handle_client(
        stream: &mut TcpStream,
        expected_token: &Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let active_jobs = AtomicUsize::new(0);
        let vfs = Arc::new(VirtualFileSystem::new(1024 * 1024 * 100));
        Self::handle_connection(
            stream,
            expected_token,
            "forge-worker-node",
            8,
            &active_jobs,
            Instant::now(),
            vfs,
        )
    }

    pub fn handle_connection(
        stream: &mut TcpStream,
        expected_token: &Option<String>,
        worker_name: &str,
        max_concurrency: usize,
        active_jobs: &AtomicUsize,
        start_time: Instant,
        vfs: Arc<VirtualFileSystem>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(300)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(300)));

        let mut reader = BufReader::new(stream.try_clone()?);
        let mut line = String::new();

        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        if let Ok(req) = serde_json::from_str::<RemoteTaskRequest>(trimmed) {
            if let Some(expected) = expected_token
                && req.auth_token.as_ref() != Some(expected)
            {
                let err_res = RemoteTaskResponse {
                    task_id: req.task_id,
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: "unauthorized: invalid auth token".to_string(),
                    duration_ms: 0,
                    error: Some("unauthorized".to_string()),
                };
                let out = serde_json::to_string(&err_res)?;
                stream.write_all(out.as_bytes())?;
                stream.write_all(b"\n")?;
                stream.flush()?;
                return Ok(());
            }

            active_jobs.fetch_add(1, Ordering::SeqCst);
            let start = Instant::now();

            let mut spec = CommandSpec::new(&req.command);
            spec.args = req.args;
            for (k, v) in req.env {
                spec.env.insert(k, v);
            }

            // Materialize the packed source snapshot, if any, and re-resolve
            // the requested working directory inside it.
            let source_dir = req
                .source
                .as_ref()
                .map(|ctx| {
                    if ctx.use_vfs.unwrap_or(false) {
                        unpack_source_to_vfs(ctx, &vfs)
                    } else {
                        unpack_source(ctx)
                    }
                })
                .transpose()?;
            if let Some(ref root) = source_dir {
                spec.cwd = Some(match req.cwd.as_ref().map(Path::new) {
                    Some(cwd) => cwd
                        .strip_prefix(&req.source.as_ref().expect("source_dir implies source").root)
                        .map(|relative| root.join(relative))
                        .unwrap_or_else(|_| root.clone()),
                    None => root.clone(),
                });
            } else if let Some(cwd) = req.cwd {
                spec.cwd = Some(std::path::PathBuf::from(cwd));
            }

            let task = Task::new(&req.task_id, spec.command_line(), spec);
            let timeout = req.timeout_secs.map(Duration::from_secs);
            let executor = ProcessExecutor::with_timeout(false, timeout);
            let outcome = executor.execute(&task);

            if let Some(dir) = source_dir {
                let _ = std::fs::remove_dir_all(dir);
            }

            active_jobs.fetch_sub(1, Ordering::SeqCst);

            let duration_ms = start.elapsed().as_millis() as u64;

            let response = match outcome {
                Ok(out) => RemoteTaskResponse {
                    task_id: req.task_id,
                    exit_code: out.exit_code.or(
                        if out.status == forge_executor::TaskStatus::Failed {
                            Some(1)
                        } else {
                            Some(0)
                        },
                    ),
                    stdout: out.stdout,
                    stderr: out.stderr,
                    duration_ms,
                    error: None,
                },
                Err(e) => RemoteTaskResponse {
                    task_id: req.task_id,
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: e.to_string(),
                    duration_ms,
                    error: Some(e.to_string()),
                },
            };

            let out = serde_json::to_string(&response)?;
            stream.write_all(out.as_bytes())?;
            stream.write_all(b"\n")?;
            stream.flush()?;

            return Ok(());
        }

        if let Ok(vfs_req) = serde_json::from_str::<VfsFileRequest>(trimmed) {
            if let Some(expected) = expected_token
                && vfs_req.auth_token.as_ref() != Some(expected)
            {
                let err_resp = VfsFileResponse {
                    success: false,
                    content_base64: None,
                    error: Some("unauthorized".to_string()),
                    metadata: None,
                };
                let out = serde_json::to_string(&err_resp)?;
                stream.write_all(out.as_bytes())?;
                stream.write_all(b"\n")?;
                stream.flush()?;
                return Ok(());
            }

            let file_path = Path::new(&vfs_req.file_path);
            let response = match vfs.read_file(file_path) {
                Ok(content) => {
                    let metadata = vfs.metadata(file_path).ok();
                    VfsFileResponse {
                        success: true,
                        content_base64: Some(
                            base64::engine::general_purpose::STANDARD.encode(&content),
                        ),
                        error: None,
                        metadata: metadata.map(|m| crate::protocol::VfsFileMetadata {
                            size: m.size,
                            modified: m.modified,
                            is_executable: m.is_executable,
                        }),
                    }
                }
                Err(e) => VfsFileResponse {
                    success: false,
                    content_base64: None,
                    error: Some(e.to_string()),
                    metadata: None,
                },
            };

            let out = serde_json::to_string(&response)?;
            stream.write_all(out.as_bytes())?;
            stream.write_all(b"\n")?;
            stream.flush()?;
            return Ok(());
        }

        if let Ok(ping_req) = serde_json::from_str::<WorkerPingRequest>(trimmed) {
            if let Some(expected) = expected_token
                && ping_req.auth_token.as_ref() != Some(expected)
            {
                let current_jobs = active_jobs.load(Ordering::SeqCst);
                let cpu_est = if max_concurrency > 0 {
                    Some(((current_jobs as f32) / (max_concurrency as f32) * 100.0).min(100.0))
                } else {
                    None
                };
                let mem_est = Some((current_jobs as u64) * 64 * 1024 * 1024);

                let err_resp = WorkerPingResponse {
                    status: "unauthorized".to_string(),
                    health: WorkerHealthInfo {
                        worker_name: worker_name.to_string(),
                        active_jobs: current_jobs,
                        max_concurrency,
                        uptime_secs: start_time.elapsed().as_secs(),
                        cpu_usage_pct: cpu_est,
                        memory_used_bytes: mem_est,
                        memory_total_bytes: Some(1024 * 1024 * 1024 * 8),
                    },
                    error: Some("invalid authentication token".to_string()),
                };
                let out = serde_json::to_string(&err_resp)?;
                stream.write_all(out.as_bytes())?;
                stream.write_all(b"\n")?;
                stream.flush()?;
                return Ok(());
            }

            let current_jobs = active_jobs.load(Ordering::SeqCst);
            let cpu_est = if max_concurrency > 0 {
                Some(((current_jobs as f32) / (max_concurrency as f32) * 100.0).min(100.0))
            } else {
                None
            };
            let mem_est = Some((current_jobs as u64) * 64 * 1024 * 1024);

            let resp = WorkerPingResponse {
                status: "ok".to_string(),
                health: WorkerHealthInfo {
                    worker_name: worker_name.to_string(),
                    active_jobs: current_jobs,
                    max_concurrency,
                    uptime_secs: start_time.elapsed().as_secs(),
                    cpu_usage_pct: cpu_est,
                    memory_used_bytes: mem_est,
                    memory_total_bytes: Some(1024 * 1024 * 1024 * 8),
                },
                error: None,
            };
            let out = serde_json::to_string(&resp)?;
            stream.write_all(out.as_bytes())?;
            stream.write_all(b"\n")?;
            stream.flush()?;
            return Ok(());
        }

        let err_res = RemoteTaskResponse {
            task_id: "unknown".to_string(),
            exit_code: Some(1),
            stdout: String::new(),
            stderr: "invalid request wire format".to_string(),
            duration_ms: 0,
            error: Some("invalid_request".to_string()),
        };
        let out = serde_json::to_string(&err_res)?;
        stream.write_all(out.as_bytes())?;
        stream.write_all(b"\n")?;
        stream.flush()?;

        Ok(())
    }
}

/// Decodes and extracts a packed source snapshot into a fresh temp dir.
fn unpack_source(ctx: &SourceContext) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if ctx.format != "tar.zst" {
        return Err(format!("unsupported source format: {}", ctx.format).into());
    }
    let mut decoder = base64::read::DecoderReader::new(
        ctx.data_base64.as_bytes(),
        &base64::engine::general_purpose::STANDARD,
    );
    let mut blob = Vec::new();
    decoder.read_to_end(&mut blob)?;

    let root = tempfile::Builder::new()
        .prefix("forge-source-")
        .tempdir()
        .map_err(|e| e.to_string())?
        .keep();
    unpack_artifacts(&blob, &root).map_err(|e| e.to_string())?;
    Ok(root)
}

/// Decodes and extracts a packed source snapshot into VFS for on-demand streaming
fn unpack_source_to_vfs(
    ctx: &SourceContext,
    vfs: &VirtualFileSystem,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if ctx.format != "tar.zst" {
        return Err(format!("unsupported source format: {}", ctx.format).into());
    }

    let mut decoder = base64::read::DecoderReader::new(
        ctx.data_base64.as_bytes(),
        &base64::engine::general_purpose::STANDARD,
    );
    let mut blob = Vec::new();
    decoder.read_to_end(&mut blob)?;

    // First extract to temp directory
    let temp_root = tempfile::Builder::new()
        .prefix("forge-source-vfs-")
        .tempdir()
        .map_err(|e| e.to_string())?
        .keep();
    unpack_artifacts(&blob, &temp_root).map_err(|e| e.to_string())?;

    // Mount to VFS
    let vfs_mount = ctx.vfs_mount.as_deref().unwrap_or("/vfs");
    let vfs_path = Path::new(vfs_mount);
    vfs.mount_local(&temp_root, vfs_path)
        .map_err(|e| e.to_string())?;

    Ok(temp_root)
}
