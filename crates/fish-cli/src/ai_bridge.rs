//! Bridge between the Rust CLI and the Python AI services.
//!
//! The Python layer (`py/fish_ai/server.py`) exposes a newline-delimited
//! JSON-RPC 2.0 interface over stdio. `AiBridge` spawns that server as a
//! subprocess for a single request and returns the parsed result, giving the
//! CLI a dependency-free way to reach `analyze_failure`, `doctor_advice`, and
//! the other AI helpers without linking a Python runtime.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

pub struct AiBridge {
    /// Command (argv) used to launch the AI server. Defaults to
    /// `python3 -m fish_ai.server` and can be overridden with `FISH_AI_SERVER`.
    server_command: Vec<String>,
}

impl AiBridge {
    pub fn from_env() -> Self {
        let configured = std::env::var("FISH_AI_SERVER")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let server_command = match configured {
            Some(cmd) => split_command(&cmd),
            None => vec![
                "python3".to_string(),
                "-m".to_string(),
                "fish_ai.server".to_string(),
            ],
        };

        Self { server_command }
    }

    /// Issue a single JSON-RPC request. Returns the `result` field on success
    /// or a human-readable error message.
    pub fn request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let (program, args) = match self.server_command.split_first() {
            Some((program, args)) => (program, args),
            None => return Err("AI server command is empty".to_string()),
        };

        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to spawn AI server `{program}`: {e}"))?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        {
            let mut stdin = child.stdin.take().ok_or("AI server stdin unavailable")?;
            writeln!(stdin, "{request}").map_err(|e| format!("failed to write AI request: {e}"))?;
            // Closing stdin signals EOF: the server processes the line, replies,
            // then exits its read loop.
        }

        let stdout = child.stdout.take().ok_or("AI server stdout unavailable")?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("failed to read AI response: {e}"))?;
        let _ = child.wait();

        let value: serde_json::Value =
            serde_json::from_str(line.trim()).map_err(|e| format!("invalid AI response: {e}"))?;

        if let Some(error) = value.get("error") {
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown AI error");
            return Err(message.to_string());
        }

        Ok(value
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }
}

/// Split a command string on whitespace, honouring double quotes, so
/// `FISH_AI_SERVER="python3 -m fish_ai.server"` works as expected.
fn split_command(cmd: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in cmd.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_command_handles_plain_and_quoted_arguments() {
        assert_eq!(
            split_command("python3 -m fish_ai.server"),
            vec!["python3", "-m", "fish_ai.server"]
        );
        assert_eq!(
            split_command("\"python 3\" -m mod"),
            vec!["python 3", "-m", "mod"]
        );
        assert_eq!(split_command("   "), Vec::<String>::new());
    }
}
