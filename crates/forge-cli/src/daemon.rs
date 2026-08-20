use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[derive(Debug)]
pub struct ForgeDaemon {
    port: u16,
    running: Arc<AtomicBool>,
}

impl ForgeDaemon {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_alive(port: u16) -> bool {
        Self::send_command(port, "PING").is_ok()
    }

    pub fn send_command(port: u16, cmd: &str) -> std::io::Result<String> {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        stream.write_all(cmd.as_bytes())?;
        stream.write_all(b"\n")?;
        stream.flush()?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;
        Ok(response)
    }

    pub fn start_in_background(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))?;
        listener.set_nonblocking(true)?;
        self.running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.running);

        std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let mut reader = BufReader::new(
                            stream
                                .try_clone()
                                .unwrap_or_else(|_| stream.try_clone().unwrap()),
                        );
                        let mut line = String::new();
                        if reader.read_line(&mut line).is_ok() {
                            let trimmed = line.trim();
                            if trimmed == "PING" {
                                let _ = stream.write_all(b"PONG\n");
                                let _ = stream.flush();
                            } else if trimmed == "STATUS" {
                                let _ = stream.write_all(b"FORGE_DAEMON_OK:WARMED\n");
                                let _ = stream.flush();
                            } else if trimmed == "SHUTDOWN" {
                                let _ = stream.write_all(b"FORGE_DAEMON_STOPPING\n");
                                let _ = stream.flush();
                                running.store(false, Ordering::SeqCst);
                                break;
                            } else {
                                let _ = stream.write_all(b"UNKNOWN_CMD\n");
                                let _ = stream.flush();
                            }
                        }
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_loopback_ipc() {
        let port = 19854;
        let daemon = ForgeDaemon::new(port);
        daemon.start_in_background().unwrap();

        std::thread::sleep(Duration::from_millis(80));
        assert!(ForgeDaemon::is_alive(port));

        let pong = ForgeDaemon::send_command(port, "PING").unwrap();
        assert_eq!(pong.trim(), "PONG");

        let status = ForgeDaemon::send_command(port, "STATUS").unwrap();
        assert!(status.contains("FORGE_DAEMON_OK"));

        let _ = ForgeDaemon::send_command(port, "SHUTDOWN");
        daemon.stop();
    }
}
