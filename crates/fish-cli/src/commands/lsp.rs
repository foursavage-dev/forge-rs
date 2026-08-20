use std::io::{self, BufRead, Write};
use std::process::ExitCode;

pub fn run_lsp() -> ExitCode {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();

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

        let len_str = line_trimmed
            .trim_start_matches("Content-Length:")
            .trim();
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

        let response = handle_request(method, &json_val, id);
        let Ok(resp_bytes) = serde_json::to_vec(&response) else {
            continue;
        };
        let header = format!("Content-Length: {}\r\n\r\n", resp_bytes.len());

        let _ = stdout.write_all(header.as_bytes());
        let _ = stdout.write_all(&resp_bytes);
        let _ = stdout.flush();
    }

    ExitCode::SUCCESS
}

fn get_hover_documentation(key: &str) -> &'static str {
    match key {
        "backend" => "**backend** (`string`)\n\nPrimary toolchain adapter for the package or workspace (e.g. `rust`, `go`, `ts`, `py`, `cc`, `docker`).",
        "jobs" => "**jobs** (`integer`)\n\nMaximum concurrent worker tasks. Defaults to system logical CPU count.",
        "no_cache" => "**no_cache** (`boolean`)\n\nWhen true, bypasses both local and remote Content-Addressable Storage (CAS) caches.",
        "sandbox" => "**sandbox** (`boolean`)\n\nEnables process isolation using Linux Bubblewrap, macOS sandbox-exec, or Windows Job Objects.",
        "semantic" => "**semantic** (`boolean`)\n\nEnables AST-level semantic change detection to avoid rebuilding downstream packages when public interface is unchanged.",
        "critical_path" => "**critical_path** (`boolean`)\n\nPrioritizes tasks along the longest dependency chain in the DAG.",
        "ram_limit" => "**ram_limit** (`integer 1-100`)\n\nMemory usage threshold percentage to dynamically throttle concurrency and prevent OOM.",
        "dir" => "**dir** (`string`)\n\nPath to the Content-Addressable Storage (CAS) directory (defaults to `~/.fish/cache`).",
        "reflink" => "**reflink** (`boolean`)\n\nEnables Copy-on-Write (CoW) extents or hardlinks to materialize cached artifacts with 0ms I/O copy.",
        "cache_url" => "**cache_url** (`string`)\n\nHTTP/gRPC endpoint for remote artifact cache server.",
        "token" => "**token** (`string`)\n\nBearer authentication token for remote operations.",
        "workers" => "**workers** (`array of strings`)\n\nRemote cluster worker node endpoints (e.g. `[\"worker1:9000\", \"worker2:9000\"]`).",
        "port" => "**port** (`integer`)\n\nLoopback TCP port for Fish background build daemon (default `9527`).",
        "depends_on" => "**depends_on** (`array of strings`)\n\nList of prerequisite tasks required before this pipeline stage executes.",
        "inputs" => "**inputs** (`array of globs`)\n\nFile patterns included in the task fingerprint hash computation.",
        "outputs" => "**outputs** (`array of globs`)\n\nArtifact file patterns produced and saved into the CAS cache.",
        _ => "### Fish Manifest Configuration Key\nConfiguration property for the Fish build orchestration system.",
    }
}

fn handle_request(
    method: &str,
    req: &serde_json::Value,
    id: Option<&serde_json::Value>,
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
            let position = req.get("params").and_then(|p| p.get("position"));
            let line_num = position.and_then(|pos| pos.get("line")).and_then(|l| l.as_u64()).unwrap_or(0);
            
            let hover_text = if line_num == 0 {
                "### [build] Configuration Section\nGlobal execution and toolchain configuration."
            } else {
                get_hover_documentation("backend")
            };

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
                    "label": "backend",
                    "kind": 10,
                    "detail": "Toolchain identifier (rust, go, ts, py, cc, docker, java, dotnet)",
                    "insertText": "backend = \"rust\""
                }),
                serde_json::json!({
                    "label": "jobs",
                    "kind": 10,
                    "detail": "Max worker tasks",
                    "insertText": "jobs = 8"
                }),
                serde_json::json!({
                    "label": "no_cache",
                    "kind": 10,
                    "detail": "Disable cache",
                    "insertText": "no_cache = false"
                }),
                serde_json::json!({
                    "label": "sandbox",
                    "kind": 10,
                    "detail": "Enable process sandbox",
                    "insertText": "sandbox = false"
                }),
                serde_json::json!({
                    "label": "semantic",
                    "kind": 10,
                    "detail": "Enable AST semantic invalidation",
                    "insertText": "semantic = true"
                }),
                serde_json::json!({
                    "label": "critical_path",
                    "kind": 10,
                    "detail": "Prioritize dependency critical path",
                    "insertText": "critical_path = true"
                }),
                serde_json::json!({
                    "label": "ram_limit",
                    "kind": 10,
                    "detail": "RAM concurrency governor threshold",
                    "insertText": "ram_limit = 85"
                }),
                serde_json::json!({
                    "label": "pipelines.build",
                    "kind": 7,
                    "detail": "Build pipeline stage definition",
                    "insertText": "[pipelines.build]\ndepends_on = [\"^build\"]\ninputs = [\"src/**/*\"]\noutputs = [\"target/release/*\"]"
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
