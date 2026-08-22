#![forbid(unsafe_code)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::api::{ApiState, handle_api_request};

const INDEX_HTML: &str = include_str!("../static/index.html");

pub struct DashboardServer {
    port: u16,
    state: Arc<ApiState>,
    running: Arc<AtomicBool>,
}

impl DashboardServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            state: Arc::new(ApiState::new()),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn state(&self) -> Arc<ApiState> {
        self.state.clone()
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn run_blocking(self) -> std::io::Result<()> {
        let listener = TcpListener::bind(("127.0.0.1", self.port))?;
        listener.set_nonblocking(true)?;
        self.running.store(true, Ordering::SeqCst);

        while self.running.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let state = self.state.clone();
                    thread::spawn(move || {
                        let _ = Self::handle_connection(stream, state);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    pub fn spawn(self) -> JoinHandle<std::io::Result<()>> {
        thread::spawn(move || self.run_blocking())
    }

    fn handle_connection(mut stream: TcpStream, state: Arc<ApiState>) -> std::io::Result<()> {
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        if reader.read_line(&mut request_line)? == 0 {
            return Ok(());
        }

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(());
        }

        let method = parts[0];
        let path = parts[1];

        let mut content_length = 0;
        loop {
            let mut header_line = String::new();
            if reader.read_line(&mut header_line)? == 0
                || header_line == "\r\n"
                || header_line == "\n"
            {
                break;
            }
            let lower = header_line.to_ascii_lowercase();
            if lower.starts_with("content-length:")
                && let Some(val) = lower.split(':').nth(1)
            {
                content_length = val.trim().parse().unwrap_or(0);
            }
        }

        let mut body = vec![0u8; content_length];
        if content_length > 0 {
            reader.read_exact(&mut body)?;
        }

        if path == "/" || path == "/index.html" {
            let body_bytes = INDEX_HTML.as_bytes();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
                body_bytes.len()
            );
            stream.write_all(response.as_bytes())?;
            stream.write_all(body_bytes)?;
            stream.flush()?;
        } else if path.starts_with("/api/") {
            let (status, content_type, response_body) =
                handle_api_request(&state, method, path, &body);
            let status_text = match status {
                200 => "200 OK",
                400 => "400 Bad Request",
                404 => "404 Not Found",
                _ => "500 Internal Server Error",
            };
            let response = format!(
                "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nConnection: close\r\n\r\n",
                status_text,
                content_type,
                response_body.len()
            );
            stream.write_all(response.as_bytes())?;
            stream.write_all(&response_body)?;
            stream.flush()?;
        } else {
            let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
            stream.write_all(not_found.as_bytes())?;
            stream.flush()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let dashboard = DashboardServer::new(8080);
        assert_eq!(dashboard.port, 8080);
    }
}
