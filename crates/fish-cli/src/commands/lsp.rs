use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::process::ExitCode;

/// Flat `fish.toml` configuration keys, their type, and documentation. This
/// table mirrors the actual `FishConfig` schema (`crates/fish-cli/src/config.rs`)
/// and is the single source of truth for hover and completion.
const CONFIG_KEYS: &[(&str, &str, &str)] = &[
    (
        "backend",
        "string",
        "Primary toolchain adapter for the workspace (e.g. `rust`, `go`, `ts`, `py`, `cc`, `docker`, `java`, `dotnet`, `swift`, `dart`, `zig`, `plugin`).",
    ),
    (
        "jobs",
        "integer",
        "Maximum concurrent worker tasks (0 = auto, based on logical CPU count).",
    ),
    (
        "no_cache",
        "boolean",
        "When true, bypasses both local and remote Content-Addressable Storage (CAS) caches.",
    ),
    (
        "sandbox",
        "boolean",
        "Enables process isolation using Linux Bubblewrap, macOS sandbox-exec, or Windows Job Objects.",
    ),
    ("timeout", "integer", "Per-task timeout in seconds."),
    ("profile", "string", "Named configuration profile to apply."),
    (
        "tui",
        "boolean",
        "Render the interactive terminal UI during builds.",
    ),
    (
        "remote_cache",
        "string",
        "HTTP/gRPC endpoint for the remote artifact cache server.",
    ),
    (
        "remote_cache_token",
        "string",
        "Bearer authentication token for the remote cache.",
    ),
    (
        "remote_workers",
        "array",
        "Remote cluster worker node endpoints (e.g. `[\"worker1:9000\", \"worker2:9000\"]`).",
    ),
    (
        "remote_workers_token",
        "string",
        "Bearer authentication token for remote workers.",
    ),
    (
        "cache_dir",
        "string",
        "Path to the local Content-Addressable Storage (CAS) directory (default `~/.fish/cache`).",
    ),
    (
        "send_source",
        "boolean",
        "Upload source snapshots to remote workers.",
    ),
    (
        "ram_limit",
        "integer",
        "Memory usage threshold percentage to dynamically throttle concurrency and prevent OOM.",
    ),
    (
        "semantic",
        "boolean",
        "Enables AST-level semantic change detection to avoid rebuilding downstream packages when the public interface is unchanged.",
    ),
    (
        "ramdisk",
        "boolean",
        "Materialize the local cache on a RAM disk for faster I/O.",
    ),
    (
        "swarm",
        "boolean",
        "Enable LAN peer discovery for distributed caching.",
    ),
    (
        "reflink",
        "boolean",
        "Enables Copy-on-Write (CoW) materialization of cached artifacts.",
    ),
    (
        "hermetic_trace",
        "boolean",
        "Trace file access to detect undeclared inputs/outputs.",
    ),
    (
        "swarm_compute",
        "boolean",
        "Advertise this machine as a swarm compute worker.",
    ),
    (
        "critical_path",
        "boolean",
        "Prioritizes tasks along the longest dependency chain in the DAG.",
    ),
    (
        "turbo_link",
        "boolean",
        "Use a fast linker (mold/lld) when available.",
    ),
    (
        "speculative",
        "boolean",
        "Enable speculative pre-compilation on idle cores.",
    ),
    ("daemon_pool", "boolean", "Use the compiler daemon pool."),
    (
        "kernel_bypass",
        "boolean",
        "Enable kernel-bypass experimental I/O.",
    ),
    (
        "wasm_sandbox",
        "boolean",
        "Run plugins inside a Wasm sandbox.",
    ),
    ("super_opt", "boolean", "Enable binary super-optimization."),
];

pub fn run_lsp() -> ExitCode {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();

    // Full-sync documents keyed by URI, used for hover and diagnostics.
    let mut documents: HashMap<String, String> = HashMap::new();

    loop {
        let mut line = String::new();
        let Ok(bytes_read) = reader.read_line(&mut line) else {
            break;
        };
        if bytes_read == 0 {
            break;
        }

        let line_trimmed = line.trim();
        if !line_trimmed.starts_with("Content-Length:") {
            continue;
        }

        let len_str = line_trimmed.trim_start_matches("Content-Length:").trim();
        let content_length: usize = len_str.parse().unwrap_or(0);

        let mut empty_line = String::new();
        if reader.read_line(&mut empty_line).is_err() {
            break;
        }

        let mut content_buf = vec![0u8; content_length];
        if io::Read::read_exact(&mut reader, &mut content_buf).is_err() {
            break;
        }

        let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&content_buf) else {
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

        let messages = handle_request(method, &json_val, id, &mut documents);
        for message in messages {
            let Ok(resp_bytes) = serde_json::to_vec(&message) else {
                continue;
            };
            let header = format!("Content-Length: {}\r\n\r\n", resp_bytes.len());

            let _ = stdout.write_all(header.as_bytes());
            let _ = stdout.write_all(&resp_bytes);
            let _ = stdout.flush();
        }
    }

    ExitCode::SUCCESS
}

fn get_hover_documentation(key: &str) -> String {
    for (name, ty, doc) in CONFIG_KEYS {
        if *name == key {
            return format!("**{name}** (`{ty}`)\n\n{doc}");
        }
    }
    format!(
        "### Fish Manifest Configuration Key\n`{key}` is not a known `fish.toml` key. See `fish --help` for the supported schema."
    )
}

/// Extract the configuration key present on `line` (the text before `=`).
fn key_at_line(text: &str, line: usize) -> Option<String> {
    let target = text.lines().nth(line)?;
    let trimmed = target.trim_start();
    let key: String = trimmed
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if key.is_empty() { None } else { Some(key) }
}

/// Find the 0-based line number where `key = ...` appears.
fn find_key_line(text: &str, key: &str) -> u64 {
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(key) && trimmed[key.len()..].trim_start().starts_with('=') {
            return index as u64;
        }
    }
    0
}

fn diagnostic(uri: &str, line: u64, message: &str, severity: u64) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": [{
                "range": {
                    "start": { "line": line, "character": 0 },
                    "end": { "line": line, "character": 256 }
                },
                "severity": severity,
                "source": "fish-lsp",
                "message": message
            }]
        }
    })
}

/// Validate a document's top-level keys against the known schema and return
/// publishDiagnostics notifications for unknown or malformed content.
fn validate_document(uri: &str, text: &str) -> Vec<serde_json::Value> {
    let Ok(doc) = toml::from_str::<toml::Value>(text) else {
        return vec![diagnostic(uri, 0, "invalid TOML in fish.toml", 1)];
    };

    let Some(table) = doc.as_table() else {
        return Vec::new();
    };

    let mut notifications = Vec::new();
    for key in table.keys() {
        if key == "pipelines" {
            continue;
        }
        let known = CONFIG_KEYS.iter().any(|(name, _, _)| name == key);
        if !known {
            let line = find_key_line(text, key);
            notifications.push(diagnostic(
                uri,
                line,
                &format!("unknown configuration key `{key}`"),
                2,
            ));
        }
    }
    notifications
}

/// Extract the current document text from an LSP notification parameter.
fn document_text(params: Option<&serde_json::Value>) -> Option<&str> {
    let text_document = params?.get("textDocument")?;
    // `didOpen` carries the full text in `textDocument.text`.
    if let Some(text) = text_document.get("text").and_then(|t| t.as_str()) {
        return Some(text);
    }
    // `didChange` carries it in `contentChanges[0].text`.
    params?
        .get("contentChanges")?
        .get(0)?
        .get("text")
        .and_then(|t| t.as_str())
}

fn handle_request(
    method: &str,
    req: &serde_json::Value,
    id: Option<&serde_json::Value>,
    documents: &mut HashMap<String, String>,
) -> Vec<serde_json::Value> {
    match method {
        "initialize" => vec![serde_json::json!({
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
        })],
        "shutdown" => vec![serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": null
        })],
        "textDocument/hover" => {
            let uri = req
                .get("params")
                .and_then(|p| p.get("textDocument"))
                .and_then(|td| td.get("uri"))
                .and_then(|u| u.as_str())
                .unwrap_or("");
            let line_num = req
                .get("params")
                .and_then(|p| p.get("position"))
                .and_then(|pos| pos.get("line"))
                .and_then(|l| l.as_u64())
                .unwrap_or(0) as usize;

            let hover_text = documents
                .get(uri)
                .and_then(|text| key_at_line(text, line_num))
                .map(|key| get_hover_documentation(&key))
                .unwrap_or_else(|| {
                    "### [fish.toml] Configuration\nFlat `key = value` manifest for the Fish build orchestration system.".to_string()
                });

            vec![serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "contents": {
                        "kind": "markdown",
                        "value": hover_text
                    }
                }
            })]
        }
        "textDocument/completion" => {
            let items: Vec<serde_json::Value> = CONFIG_KEYS
                .iter()
                .map(|(key, ty, doc)| {
                    let insert_text = match *ty {
                        "boolean" => format!("{key} = false"),
                        "integer" => format!("{key} = 0"),
                        "array" => format!("{key} = []"),
                        _ => format!("{key} = \"\""),
                    };
                    serde_json::json!({
                        "label": key,
                        "kind": 10,
                        "detail": *ty,
                        "documentation": *doc,
                        "insertText": insert_text
                    })
                })
                .collect();
            vec![serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": items
            })]
        }
        "textDocument/didOpen" | "textDocument/didChange" | "textDocument/didSave" => {
            let params = req.get("params");
            let uri = params
                .and_then(|p| p.get("textDocument"))
                .and_then(|td| td.get("uri"))
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .to_string();

            if let Some(text) = document_text(params) {
                documents.insert(uri.clone(), text.to_string());
                return validate_document(&uri, text);
            }
            Vec::new()
        }
        _ => vec![serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": null
        })],
    }
}
