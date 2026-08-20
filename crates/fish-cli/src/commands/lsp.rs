use std::io::{self, BufRead, Read, Write};
use std::process::ExitCode;

pub fn run_lsp() -> ExitCode {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin.lock());
    let mut stdout = io::stdout().lock();

    let mut line_buf = String::new();

    loop {
        line_buf.clear();
        let mut content_length: usize = 0;

        loop {
            line_buf.clear();
            let bytes_read = match reader.read_line(&mut line_buf) {
                Ok(n) => n,
                Err(_) => return ExitCode::FAILURE,
            };

            if bytes_read == 0 {
                return ExitCode::SUCCESS;
            }

            let trimmed = line_buf.trim();
            if trimmed.is_empty() {
                break;
            }

            if let Some(val) = trimmed.strip_prefix("Content-Length:") {
                if let Ok(len) = val.trim().parse::<usize>() {
                    content_length = len;
                }
            }
        }

        if content_length == 0 {
            continue;
        }

        let mut body = vec![0u8; content_length];
        if reader.read_exact(&mut body).is_err() {
            break;
        }

        let Ok(req_str) = std::str::from_utf8(&body) else {
            continue;
        };

        let Ok(json_val) = serde_json::from_str::<serde_json::Value>(req_str) else {
            continue;
        };

        let method = json_val
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let id = json_val.get("id");

        if method == "exit" {
            break;
        }

        if let Some(id_val) = id {
            let response = handle_lsp_request(method, &json_val, id_val);
            let resp_bytes = serde_json::to_vec(&response).unwrap_or_default();
            let header = format!("Content-Length: {}\r\n\r\n", resp_bytes.len());
            let _ = stdout.write_all(header.as_bytes());
            let _ = stdout.write_all(&resp_bytes);
            let _ = stdout.flush();
        }
    }

    ExitCode::SUCCESS
}

fn handle_lsp_request(
    method: &str,
    _req: &serde_json::Value,
    id: &serde_json::Value,
) -> serde_json::Value {
    match method {
        "initialize" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "capabilities": {
                    "textDocumentSync": 1,
                    "completionProvider": {
                        "resolveProvider": false,
                        "triggerCharacters": ["[", "=", ".", "\""]
                    },
                    "hoverProvider": true
                },
                "serverInfo": {
                    "name": "fish-lsp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        }),
        "shutdown" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": null
        }),
        "textDocument/hover" => {
            let hover_text =
                "### Fish Manifest Property\nFish build orchestration configuration key.";
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "contents": {
                        "kind": "markdown",
                        "value": hover_text
                    }
                }
            })
        }
        "textDocument/completion" => {
            let items = vec![
                serde_json::json!({
                    "label": "workspace",
                    "kind": 7,
                    "detail": "Define workspace members and root properties"
                }),
                serde_json::json!({
                    "label": "backend",
                    "kind": 7,
                    "detail": "Specify compiler backend (rust, go, ts, py, cc, etc.)"
                }),
                serde_json::json!({
                    "label": "cache",
                    "kind": 7,
                    "detail": "Configure L1/L2 CAS caching rules"
                }),
                serde_json::json!({
                    "label": "ai",
                    "kind": 7,
                    "detail": "Configure AI failure diagnostics and scheduler optimization"
                }),
            ];
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": items
            })
        }
        _ => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": null
        }),
    }
}
