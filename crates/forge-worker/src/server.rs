use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crate::protocol::{RemoteTaskRequest, RemoteTaskResponse};
use forge_executor::{CommandSpec, ProcessExecutor, Task, TaskExecutor};

pub struct WorkerServer {
    addr: String,
    auth_token: Option<String>,
    running: Arc<AtomicBool>,
}

impl WorkerServer {
    pub fn new(addr: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            addr: addr.into(),
            auth_token,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }

    pub fn handle_client(
        stream: &mut TcpStream,
        expected_token: &Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }

        let req: RemoteTaskRequest = serde_json::from_str(&line)?;

        if let Some(expected) = expected_token {
            if req.auth_token.as_ref() != Some(expected) {
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
        }

        let start = Instant::now();
        let mut spec = CommandSpec::new(&req.command);
        for arg in &req.args {
            spec = spec.arg(arg);
        }
        for (k, v) in &req.env {
            spec = spec.env(k, v);
        }
        if let Some(ref cwd) = req.cwd {
            spec = spec.cwd(cwd);
        }

        let task = Task::new(&req.task_id, spec.command_line(), spec);
        let executor = ProcessExecutor::new(false);
        let outcome = executor.execute(&task);

        let duration_ms = start.elapsed().as_millis() as u64;

        let response = match outcome {
            Ok(res) => RemoteTaskResponse {
                task_id: req.task_id,
                exit_code: res.exit_code,
                stdout: res.stdout,
                stderr: res.stderr,
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
        Ok(())
    }

    pub fn run_loop(&self, listener: TcpListener) {
        self.running.store(true, Ordering::SeqCst);
        for stream in listener.incoming() {
            if !self.running.load(Ordering::SeqCst) {
                break;
            }
            if let Ok(mut s) = stream {
                let _ = Self::handle_client(&mut s, &self.auth_token);
            }
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
