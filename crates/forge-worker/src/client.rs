use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::protocol::{RemoteTaskRequest, RemoteTaskResponse};
use forge_executor::{ExecutorError, Task, TaskExecutor, TaskOutcome, TaskStatus};

#[derive(Debug, Clone)]
pub struct RemoteWorkerClient {
    pub server_addr: String,
    pub auth_token: Option<String>,
    pub timeout: Duration,
}

impl RemoteWorkerClient {
    pub fn new(server_addr: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            server_addr: server_addr.into(),
            auth_token,
            timeout: Duration::from_secs(60),
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

        let req = RemoteTaskRequest {
            task_id: task.label.clone(),
            command: task.spec.program.clone(),
            args: task.spec.args.clone(),
            env: env_map,
            cwd,
            auth_token: self.auth_token.clone(),
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
        reader
            .read_line(&mut line)
            .map_err(|e| ExecutorError::Spawn {
                command: task.label.clone(),
                source: e,
            })?;

        let res: RemoteTaskResponse =
            serde_json::from_str(&line).map_err(|e| ExecutorError::Spawn {
                command: task.label.clone(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            })?;

        let status = if res.exit_code == Some(0) && res.error.is_none() {
            TaskStatus::Executed
        } else {
            TaskStatus::Failed
        };

        Ok(TaskOutcome {
            status,
            exit_code: res.exit_code,
            stdout: res.stdout,
            stderr: res.stderr,
            duration: Duration::from_millis(res.duration_ms),
        })
    }
}

impl TaskExecutor for RemoteWorkerClient {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        self.send_task(task)
    }
}
