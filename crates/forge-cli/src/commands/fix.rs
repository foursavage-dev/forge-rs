#![forbid(unsafe_code)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use wait_timeout::ChildExt;

pub struct FixDiagnostics {
    pub error_count: usize,
    pub warning_count: usize,
    pub suggestions: Vec<FixSuggestion>,
    pub ai_insights: Option<String>,
}

pub struct FixSuggestion {
    pub category: String,
    pub description: String,
    pub suggested_command: Option<String>,
    pub patch: Option<String>,
}

pub fn run_fix(
    project_path: Option<PathBuf>,
    apply: bool,
    use_ai: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let root =
        project_path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    println!("🦀 Forge Auto-Healer & AI Diagnostics");
    println!("============================================================");
    println!("🔍 Inspecting project at: {}", root.display());

    let output = Command::new("cargo")
        .arg("check")
        .arg("--message-format=json")
        .current_dir(&root)
        .output();

    let mut suggestions = Vec::new();
    let mut raw_errors = Vec::new();
    let mut error_count = 0;
    let mut warning_count = 0;

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);

        for line in stdout.lines() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(reason) = val.get("reason").and_then(|r| r.as_str()) {
                    if reason == "compiler-message" {
                        if let Some(msg) = val.get("message") {
                            let level = msg.get("level").and_then(|l| l.as_str()).unwrap_or("");
                            let rendered =
                                msg.get("rendered").and_then(|r| r.as_str()).unwrap_or("");

                            if level == "error" {
                                error_count += 1;
                                raw_errors.push(rendered.to_string());

                                if let Some(code_obj) = msg.get("code") {
                                    if let Some(code) =
                                        code_obj.get("code").and_then(|c| c.as_str())
                                    {
                                        let message_text = msg
                                            .get("message")
                                            .and_then(|m| m.as_str())
                                            .unwrap_or("");
                                        let suggestion =
                                            analyze_rustc_error(code, message_text, rendered);
                                        suggestions.push(suggestion);
                                    }
                                }
                            } else if level == "warning" {
                                warning_count += 1;
                            }
                        }
                    }
                }
            }
        }

        if error_count == 0 && !out.status.success() {
            raw_errors.push(stderr.to_string());
            error_count += 1;
            suggestions.push(FixSuggestion {
                category: "Build Failure".to_string(),
                description: stderr.trim().to_string(),
                suggested_command: None,
                patch: None,
            });
        }
    }

    let mut ai_insights = None;
    if use_ai && (!raw_errors.is_empty() || !suggestions.is_empty()) {
        if let Ok(key) = env::var("GEMINI_API_KEY") {
            println!("🤖 Querying Google Gemini AI for root-cause diagnosis...");
            ai_insights = query_gemini_healer(&key, &raw_errors);
        } else {
            println!(
                "💡 Notice: Set GEMINI_API_KEY environment variable to enable deep Gemini AI root-cause analysis."
            );
        }
    }

    let diagnostics = FixDiagnostics {
        error_count,
        warning_count,
        suggestions,
        ai_insights,
    };

    render_diagnostics(&diagnostics);

    if apply {
        apply_suggestions(&diagnostics.suggestions, &root)?;
    }

    Ok(())
}

fn analyze_rustc_error(code: &str, message: &str, _rendered: &str) -> FixSuggestion {
    match code {
        "E0432" | "E0433" => {
            let crate_name = message
                .split('`')
                .nth(1)
                .unwrap_or("missing_crate")
                .split("::")
                .next()
                .unwrap_or("");
            FixSuggestion {
                category: "Unresolved Import / Missing Crate".to_string(),
                description: format!("Item or module `{}` cannot be found.", crate_name),
                suggested_command: Some(format!("cargo add {}", crate_name)),
                patch: None,
            }
        }
        "E0425" => FixSuggestion {
            category: "Unresolved Identifier".to_string(),
            description: format!("Value or function not found in scope: {}", message),
            suggested_command: None,
            patch: None,
        },
        "E0308" => FixSuggestion {
            category: "Mismatched Types".to_string(),
            description: format!("Type mismatch detected: {}", message),
            suggested_command: None,
            patch: None,
        },
        "E0599" => FixSuggestion {
            category: "No Method or Associated Item".to_string(),
            description: format!(
                "Method not found: {}. You may need to import a trait.",
                message
            ),
            suggested_command: None,
            patch: None,
        },
        _ => FixSuggestion {
            category: format!("Compiler Error [{}]", code),
            description: message.to_string(),
            suggested_command: None,
            patch: None,
        },
    }
}

fn render_diagnostics(diag: &FixDiagnostics) {
    println!();
    if diag.error_count == 0 {
        println!("✨ [Clean] No compile errors detected! Your project builds smoothly.");
        if diag.warning_count > 0 {
            println!(
                "ℹ️  {} compiler warnings found. Run `cargo clippy` or `forge check` for code hygiene.",
                diag.warning_count
            );
        }
        return;
    }

    println!(
        "🚨 Diagnostics Summary: {} errors, {} warnings found.\n",
        diag.error_count, diag.warning_count
    );

    for (idx, sugg) in diag.suggestions.iter().enumerate() {
        println!("{}. [{}] {}", idx + 1, sugg.category, sugg.description);
        if let Some(cmd) = &sugg.suggested_command {
            println!("   🛠️ Suggested Fix Command: `{}`", cmd);
        }
        if let Some(patch) = &sugg.patch {
            println!("   📝 Suggested Diff:\n{}", patch);
        }
        println!();
    }

    if let Some(insights) = &diag.ai_insights {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🧠 AI Root-Cause Diagnostic & Remediation Plan:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("{}", insights);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}

fn is_safe_command(cmd: &str) -> bool {
    // Whitelist of safe commands that can be auto-executed
    let safe_commands = ["cargo", "rustfmt", "rustup"];

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }

    let command = parts[0];

    // Check if the base command is in the whitelist
    if !safe_commands.contains(&command) {
        return false;
    }

    // Additional safety checks for specific commands
    if command == "cargo" {
        // Only allow safe cargo subcommands
        let safe_cargo_subcommands = [
            "add", "fmt", "clippy", "check", "test", "build", "doc", "clean", "update",
        ];
        if parts.len() > 1 {
            let subcommand = parts[1];
            if !safe_cargo_subcommands.contains(&subcommand) {
                return false;
            }
        }
    }

    // Reject commands with shell operators to prevent injection
    if cmd.contains(';') || cmd.contains('&') || cmd.contains('|') || cmd.contains('$') {
        return false;
    }

    // Reject commands with backticks (command substitution)
    if cmd.contains('`') {
        return false;
    }

    // Reject commands with redirects (prevent file manipulation)
    if cmd.contains('>') || cmd.contains('<') {
        return false;
    }

    true
}

fn apply_suggestions(
    suggestions: &[FixSuggestion],
    root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut applied_count = 0;
    for sugg in suggestions {
        if let Some(cmd_str) = &sugg.suggested_command {
            println!("⚡ Checking safety of command: {}", cmd_str);

            if !is_safe_command(cmd_str) {
                println!("⚠️  Command blocked for security reasons: {}", cmd_str);
                println!("    Only whitelisted development tools can be auto-executed.");
                continue;
            }

            println!("⚡ Executing safe command: {}", cmd_str);
            let parts: Vec<&str> = cmd_str.split_whitespace().collect();
            if !parts.is_empty() {
                match Command::new(parts[0])
                    .args(&parts[1..])
                    .current_dir(root)
                    .spawn()
                {
                    Ok(mut child) => match child.wait_timeout(Duration::from_secs(120)) {
                        Ok(Some(st)) => {
                            if st.success() {
                                applied_count += 1;
                            } else {
                                println!("⚠️  Command failed with status: {:?}", st);
                            }
                        }
                        Ok(None) => {
                            let _ = child.kill();
                            let _ = child.wait();
                            println!(
                                "⚠️  Command timed out after 120s and was killed: {}",
                                cmd_str
                            );
                        }
                        Err(e) => {
                            println!("⚠️  Failed to wait for command: {} ({})", cmd_str, e);
                        }
                    },
                    Err(e) => {
                        println!("⚠️  Failed to execute command: {} ({})", cmd_str, e);
                    }
                }
            }
        }
    }
    if applied_count > 0 {
        println!(
            "✅ Successfully applied {} auto-fix remediation actions.",
            applied_count
        );
    }
    Ok(())
}

fn query_gemini_healer(api_key: &str, raw_errors: &[String]) -> Option<String> {
    let joined_errors = raw_errors.join("\n").chars().take(4000).collect::<String>();
    let prompt = format!(
        "You are an elite Rust systems engineer and build diagnostic AI for Forge build tool. Analyze these build errors and provide a concise, high-impact root cause diagnosis and direct code solution:\n\n```\n{}\n```",
        joined_errors
    );

    let payload = serde_json::json!({
        "contents": [{
            "parts": [{ "text": prompt }]
        }]
    })
    .to_string();

    let url =
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent";

    let output = Command::new("curl")
        .arg("-sS")
        .arg("--max-time")
        .arg("15")
        .arg("-X")
        .arg("POST")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-H")
        .arg(format!("x-goog-api-key: {}", api_key))
        .arg("-d")
        .arg(&payload)
        .arg(url)
        .output()
        .ok()?;

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(res) = val
                .pointer("/candidates/0/content/parts/0/text")
                .and_then(|t| t.as_str())
            {
                return Some(res.to_string());
            }
        }
    }
    None
}
