use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Background build daemon exposing a JSON-RPC 2.0 interface over
/// newline-delimited messages.
///
/// On Unix the primary transport is a Unix domain socket (sub-millisecond
/// local IPC); TCP on `127.0.0.1` is kept as a fallback and is the only
/// transport on Windows.
#[derive(Debug)]
pub struct FishDaemon {
    port: u16,
    running: Arc<AtomicBool>,
}

impl FishDaemon {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Deterministic per-port Unix socket path used by both the daemon and
    /// its clients.
    #[cfg(unix)]
    fn socket_path(port: u16) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("fish-daemon-{port}.sock"))
    }

    pub fn is_alive(port: u16) -> bool {
        Self::send_command(port, "ping").is_ok()
    }

    fn jsonrpc_request(id: u64, method: &str) -> String {
        serde_json::json!({ "jsonrpc": "2.0", "id": id, "method": method }).to_string()
    }

    fn jsonrpc_result(id: u64, result: &str) -> String {
        serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
    }

    /// Dispatch a method to its result. Returns `(shutdown_requested, result)`.
    fn dispatch_method(method: &str) -> (bool, String) {
        match method.to_ascii_lowercase().as_str() {
            "ping" => (false, "PONG".to_string()),
            "status" => (
                false,
                format!("FISH_DAEMON_OK:WARMED:pid={}", std::process::id()),
            ),
            "shutdown" => (true, "FISH_DAEMON_STOPPING".to_string()),
            other => (false, format!("error: unknown method `{other}`")),
        }
    }

    /// Parse an incoming request line. Plain-text legacy commands (without a
    /// JSON body) are still accepted as a method name with id 0.
    fn parse_request(line: &str) -> (u64, String) {
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(req) => (
                req.get("id").and_then(|i| i.as_u64()).unwrap_or(0),
                req.get("method")
                    .and_then(|m| m.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            Err(_) => (0, line.to_string()),
        }
    }

    fn round_trip(
        stream: &mut (impl std::io::Read + std::io::Write),
        line: &str,
    ) -> std::io::Result<String> {
        stream.write_all(line.as_bytes())?;
        stream.write_all(b"\n")?;
        stream.flush()?;
        let mut response = String::new();
        BufReader::new(stream).read_line(&mut response)?;
        Ok(response)
    }

    fn send_raw(port: u16, line: &str) -> std::io::Result<String> {
        #[cfg(unix)]
        {
            use std::os::unix::net::UnixStream;
            let sock = Self::socket_path(port);
            if let Ok(mut stream) = UnixStream::connect(&sock) {
                stream.set_read_timeout(Some(Duration::from_secs(2)))?;
                stream.set_write_timeout(Some(Duration::from_secs(2)))?;
                if let Ok(response) = Self::round_trip(&mut stream, line) {
                    return Ok(response);
                }
            }
        }
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        Self::round_trip(&mut stream, line)
    }

    pub fn send_command(port: u16, cmd: &str) -> std::io::Result<String> {
        let request = Self::jsonrpc_request(1, cmd);
        let line = Self::send_raw(port, &request)?;
        Ok(parse_response(&line))
    }

    pub fn start_in_background(&self) -> std::io::Result<()> {
        self.running.store(true, Ordering::SeqCst);

        #[cfg(unix)]
        self.start_unix_listener();
        self.start_tcp_listener()
    }

    #[cfg(unix)]
    fn start_unix_listener(&self) {
        use std::os::unix::net::UnixListener;

        let sock = Self::socket_path(self.port);
        // Clear a stale socket left behind by a previous unclean shutdown.
        let _ = std::fs::remove_file(&sock);
        let Ok(listener) = UnixListener::bind(&sock) else {
            return;
        };
        let _ = listener.set_nonblocking(true);

        let running = Arc::clone(&self.running);
        let cleanup = sock.clone();
        std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let running = Arc::clone(&running);
                        std::thread::spawn(move || serve_connection(stream, &running));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
            let _ = std::fs::remove_file(&cleanup);
        });
    }

    fn start_tcp_listener(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))?;
        listener.set_nonblocking(true)?;

        let running = Arc::clone(&self.running);
        std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let running = Arc::clone(&running);
                        std::thread::spawn(move || serve_connection(stream, &running));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(15));
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        #[cfg(unix)]
        {
            let _ = std::fs::remove_file(Self::socket_path(self.port));
        }
    }
}

/// Serve a single client connection until the client disconnects or requests
/// shutdown. Reads are buffered by a `BufReader`; responses are written back
/// through the reader's underlying stream, so a single handle supports both
/// directions for both `TcpStream` and `UnixStream` (neither implements
/// `Clone`, but `BufReader::get_mut` hands back the writer).
fn serve_connection<S>(stream: S, running: &AtomicBool)
where
    S: std::io::Read + std::io::Write,
{
    let mut reader = BufReader::new(stream);

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let (id, method) = FishDaemon::parse_request(line);
        let (shutdown, result) = FishDaemon::dispatch_method(&method);
        let response = FishDaemon::jsonrpc_result(id, &result);

        let writer = reader.get_mut();
        if writer.write_all(response.as_bytes()).is_err()
            || writer.write_all(b"\n").is_err()
            || writer.flush().is_err()
        {
            break;
        }
        if shutdown {
            running.store(false, Ordering::SeqCst);
            break;
        }
    }
}

/// Convert a raw response line into the caller-facing string: a JSON-RPC
/// result is unwrapped, an error is surfaced, and a legacy plain-text reply
/// passes through untouched.
fn parse_response(line: &str) -> String {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(line.trim()) else {
        return line.trim().to_string();
    };
    if let Some(err) = value.get("error") {
        let message = err
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        return format!("error: {message}");
    }
    value
        .get("result")
        .and_then(|r| r.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_loopback_ipc() {
        let port = 19854;
        let daemon = FishDaemon::new(port);
        daemon.start_in_background().unwrap();

        std::thread::sleep(Duration::from_millis(80));
        assert!(FishDaemon::is_alive(port));

        let pong = FishDaemon::send_command(port, "PING").unwrap();
        assert_eq!(pong.trim(), "PONG");

        let status = FishDaemon::send_command(port, "STATUS").unwrap();
        assert!(status.contains("FISH_DAEMON_OK"));

        let _ = FishDaemon::send_command(port, "SHUTDOWN");
        daemon.stop();
    }

    #[test]
    fn jsonrpc_request_and_response_roundtrip() {
        let request = FishDaemon::jsonrpc_request(7, "ping");
        let (id, method) = FishDaemon::parse_request(&request);
        assert_eq!(id, 7);
        assert_eq!(method, "ping");

        let response = FishDaemon::jsonrpc_result(7, "PONG");
        assert_eq!(parse_response(&response), "PONG");
    }

    #[test]
    fn jsonrpc_error_is_surfaced() {
        let error = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": { "code": -32601, "message": "unknown method" }
        })
        .to_string();
        assert_eq!(parse_response(&error), "error: unknown method");
    }

    #[test]
    fn legacy_plain_text_response_passes_through() {
        assert_eq!(parse_response("PONG"), "PONG");
    }

    #[test]
    fn dispatch_handles_known_and_unknown_methods() {
        assert_eq!(FishDaemon::dispatch_method("ping").1, "PONG");
        assert_eq!(FishDaemon::dispatch_method("PING").1, "PONG");
        assert!(
            FishDaemon::dispatch_method("STATUS")
                .1
                .contains("FISH_DAEMON_OK")
        );
        assert!(FishDaemon::dispatch_method("shutdown").0);
        assert!(
            FishDaemon::dispatch_method("nope")
                .1
                .contains("unknown method")
        );
    }
}
