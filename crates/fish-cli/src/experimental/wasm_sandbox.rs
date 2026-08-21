#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmIsolationPolicy {
    pub allow_network: bool,
    pub read_paths: Vec<PathBuf>,
    pub write_paths: Vec<PathBuf>,
    pub max_memory_mb: usize,
    pub env_whitelist: HashMap<String, String>,
    pub fuel_limit: u64,
}

#[derive(Debug, Clone)]
pub struct WasmHeaderInfo {
    pub version: u32,
    pub section_count: usize,
    pub is_valid: bool,
}

#[derive(Debug, Clone)]
pub struct WasmExecutionReport {
    pub exit_code: i32,
    pub generated_artifacts: Vec<PathBuf>,
    pub fuel_consumed: u64,
    pub memory_allocated_pages: usize,
}

pub struct WasmPluginRunner;

impl WasmPluginRunner {
    pub fn parse_and_validate_header(bytes: &[u8]) -> io::Result<WasmHeaderInfo> {
        if bytes.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "WASM binary too short",
            ));
        }

        if &bytes[0..4] != b"\0asm" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid WASM Magic Number",
            ));
        }

        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if version != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported WASM version: {}", version),
            ));
        }

        let mut offset = 8;
        let mut section_count = 0;
        while offset < bytes.len() {
            let _section_id = bytes[offset];
            offset += 1;
            if offset >= bytes.len() {
                break;
            }
            let section_size = bytes[offset] as usize;
            offset += 1 + section_size;
            section_count += 1;
        }

        Ok(WasmHeaderInfo {
            version,
            section_count,
            is_valid: true,
        })
    }

    pub fn execute_sandboxed_plugin(
        plugin_wasm: &Path,
        policy: &WasmIsolationPolicy,
        input_args: &[String],
    ) -> io::Result<Vec<PathBuf>> {
        let report = Self::execute_sandboxed_plugin_with_report(plugin_wasm, policy, input_args)?;
        Ok(report.generated_artifacts)
    }

    pub fn execute_sandboxed_plugin_with_report(
        plugin_wasm: &Path,
        policy: &WasmIsolationPolicy,
        input_args: &[String],
    ) -> io::Result<WasmExecutionReport> {
        // There is no WASM runtime embedded; actually running the module would
        // require a wasmtime/wasmi dependency and a real sandbox. Failing
        // loudly prevents fabricating output artifacts (and a fake exit code)
        // for a plugin that never ran.
        let _ = (policy, input_args);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "WASM plugin execution is not implemented (`{}`)",
                plugin_wasm.display()
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_wasm_plugin_runner_refuses_fake_execution() {
        let temp = tempdir().unwrap();
        let wasm_file = temp.path().join("plugin.wasm");
        let out_file = temp.path().join("dist/bundle.js");

        let policy = WasmIsolationPolicy {
            allow_network: false,
            read_paths: vec![temp.path().join("src")],
            write_paths: vec![out_file.clone()],
            max_memory_mb: 64,
            env_whitelist: HashMap::new(),
            fuel_limit: 50_000,
        };

        let result = WasmPluginRunner::execute_sandboxed_plugin_with_report(
            &wasm_file,
            &policy,
            &["--minify".to_string()],
        );
        assert!(
            result.is_err(),
            "unimplemented WASM execution must fail loudly"
        );
        assert!(!out_file.exists(), "no fake output artifact may be written");
        assert!(
            !wasm_file.exists(),
            "a missing plugin must not be replaced with a fabricated stub"
        );
    }

    #[test]
    fn test_wasm_header_validation_errors() {
        let invalid_bytes = [0x00, 0x00, 0x00, 0x00];
        let res = WasmPluginRunner::parse_and_validate_header(&invalid_bytes);
        assert!(res.is_err());
    }
}
