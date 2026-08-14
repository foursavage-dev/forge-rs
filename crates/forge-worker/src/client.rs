#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;

use crate::protocol::{
    RemoteTaskRequest, RemoteTaskResponse, SourceContext, WorkerHealthInfo, WorkerPingRequest,
    WorkerPingResponse,
};
use forge_executor::{ExecutorError, Task, TaskExecutor, TaskOutcome, TaskStatus};
use forge_remote_cache::artifact::pack_tree;

/// Directories skipped when snapshotting the working tree for remote
/// execution. Most of these are build output or VCS state that the remote
/// side should rebuild itself.
pub const SOURCE_EXCLUDES: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "target",
    "node_modules",
    "dist",
    "build",
    "__pycache__",
    ".venv",
    "venv",
    ".cache",
    "obj",
];

#[derive(Debug, Clone)]
pub struct RemoteWorkerClient {
    pub server_addr: String,
    pub auth_token: Option<String>,
    pub timeout: Duration,
    pub pack_source: bool,
}

impl RemoteWorkerClient {
    pub fn new(server_addr: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            server_addr: server_addr.into(),
            auth_token,
            timeout: Duration::from_secs(120),
            pack_source: false,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send a compressed snapshot of the task's working directory along with
    /// every task, so workers that do not share the filesystem can run it.
    pub fn with_source_packaging(mut self) -> Self {
        self.pack_source = true;
        self
    }

    pub fn with_source_packaging_enabled(mut self, enabled: bool) -> Self {
        self.pack_source = enabled;
        self
    }

    pub fn ping(&self) -> Result<WorkerPingResponse, ExecutorError> {
        let mut stream =
            TcpStream::connect(&self.server_addr).map_err(|e| ExecutorError::Spawn {
                command: format!("ping to {}", self.server_addr),
                source: e,
            })?;

        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

        let ping_req = WorkerPingRequest {
            auth_token: self.auth_token.clone(),
        };

        let req_json = serde_json::to_string(&ping_req).map_err(|e| ExecutorError::Spawn {
            command: "ping serialization".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        })?;

        stream
            .write_all(req_json.as_bytes())
            .map_err(|e| ExecutorError::Spawn {
                command: format!("ping to {}", self.server_addr),
                source: e,
            })?;
        stream.write_all(b"\n").map_err(|e| ExecutorError::Spawn {
            command: format!("ping to {}", self.server_addr),
            source: e,
        })?;
        stream.flush().map_err(|e| ExecutorError::Spawn {
            command: format!("ping to {}", self.server_addr),
            source: e,
        })?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        if reader.read_line(&mut line).map_err(|e| ExecutorError::Spawn {
            command: format!("read ping from {}", self.server_addr),
            source: e,
        })? == 0
        {
            return Err(ExecutorError::Spawn {
                command: format!("ping to {}", self.server_addr),
                source: std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "empty ping response",
                ),
            });
        }

        let resp: WorkerPingResponse =
            serde_json::from_str(line.trim()).map_err(|e| ExecutorError::Spawn {
                command: "ping parse".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            })?;

        Ok(resp)
    }

    pub fn health(&self) -> Result<WorkerHealthInfo, ExecutorError> {
        let ping_resp = self.ping()?;
        if ping_resp.status == "ok" {
            Ok(ping_resp.health)
        } else {
            Err(ExecutorError::Spawn {
                command: format!("health check for {}", self.server_addr),
                source: std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    ping_resp.error.unwrap_or_else(|| "unhealthy worker".to_string()),
                ),
            })
        }
    }

    pub fn send_task(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        let mut stream =
            TcpStream::connect(&self.server_addr).map_err(|e| ExecutorError::Spawn {
                command: task.label.clone(),
                source: e,
            })?;

        let _ = stream.set_read_timeout(Some(self.timeout));
        let _ = stream.set_write_timeout(Some(self.timeout));

        let mut env_map = HashMap::new();
        for (k, v) in &task.spec.env {
            env_map.insert(k.clone(), v.clone());
        }

        let cwd = task
            .spec
            .cwd
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        let source = if self.pack_source {
            pack_source_context(&task.spec.cwd).map_err(|e| ExecutorError::Spawn {
                command: task.label.clone(),
                source: std::io::Error::other(e),
            })?
        } else {
            None
        };

        let req = RemoteTaskRequest {
            task_id: task.label.clone(),
            command: task.spec.program.clone(),
            args: task.spec.args.clone(),
            env: env_map,
            cwd,
            auth_token: self.auth_token.clone(),
            timeout_secs: Some(self.timeout.as_secs()),
            source,
        };

        let req_json = serde_json::to_string(&req).map_err(|e| ExecutorError::Spawn {
            command: task.label.clone(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        })?;

        stream
            .write_all(req_json.as_bytes())
            .map_err(|e| ExecutorError::Spawn {
                command: task.label.clone(),
                source: e,
            })?;
        stream.write_all(b"\n").map_err(|e| ExecutorError::Spawn {
            command: task.label.clone(),
            source: e,
        })?;
        stream.flush().map_err(|e| ExecutorError::Spawn {
            command: task.label.clone(),
            source: e,
        })?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        if reader.read_line(&mut line).map_err(|e| ExecutorError::Spawn {
            command: task.label.clone(),
            source: e,
        })? == 0
        {
            return Err(ExecutorError::Spawn {
                command: task.label.clone(),
                source: std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "empty response from remote worker",
                ),
            });
        }

        let resp: RemoteTaskResponse =
            serde_json::from_str(line.trim()).map_err(|e| ExecutorError::Spawn {
                command: task.label.clone(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            })?;

        let status = if resp.exit_code == Some(0) && resp.error.is_none() {
            TaskStatus::Executed
        } else {
            TaskStatus::Failed
        };

        Ok(TaskOutcome {
            status,
            exit_code: resp.exit_code,
            stdout: resp.stdout,
            stderr: resp.stderr,
            duration: Duration::from_millis(resp.duration_ms),
        })
    }
}

impl TaskExecutor for RemoteWorkerClient {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        self.send_task(task)
    }
}

fn pack_source_context(cwd: &Option<PathBuf>) -> Result<Option<SourceContext>, String> {
    use base64::Engine;

    let Some(cwd) = cwd else {
        return Ok(None);
    };
    if !cwd.is_dir() {
        return Ok(None);
    }
    let blob = pack_tree(cwd, SOURCE_EXCLUDES).map_err(|e| e.to_string())?;
    let data_base64 = base64::engine::general_purpose::STANDARD.encode(blob);
    Ok(Some(SourceContext {
        root: cwd.to_string_lossy().into_owned(),
        data_base64,
        format: "tar.zst".to_string(),
    }))
}
