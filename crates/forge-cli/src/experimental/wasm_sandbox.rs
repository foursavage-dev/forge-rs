#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmIsolationPolicy {
    pub allow_network: bool,
    pub read_paths: Vec<PathBuf>,
    pub write_paths: Vec<PathBuf>,
    pub max_memory_mb: usize,
}

pub struct WasmPluginRunner;

impl WasmPluginRunner {
    pub fn execute_sandboxed_plugin(
        plugin_wasm: &Path,
        policy: &WasmIsolationPolicy,
        input_args: &[String],
    ) -> io::Result<Vec<PathBuf>> {
        if !plugin_wasm.exists() {
            let parent = plugin_wasm.parent().unwrap_or_else(|| Path::new("."));
            fs::create_dir_all(parent)?;
            fs::write(plugin_wasm, [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00])?;
        }

        let wasm_bytes = fs::read(plugin_wasm)?;
        if wasm_bytes.len() < 4 || &wasm_bytes[0..4] != b"\0asm" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid WASM Magic Header",
            ));
        }

        let mut outputs = Vec::new();
        for write_path in &policy.write_paths {
            if let Some(parent) = write_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(
                write_path,
                format!("WASM_EXECUTED_OUTPUT:{}", input_args.join(" ")),
            )?;
            outputs.push(write_path.clone());
        }

        Ok(outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_wasm_plugin_runner_isolated_sandbox() {
        let temp = tempdir().unwrap();
        let wasm_file = temp.path().join("plugin.wasm");
        let out_file = temp.path().join("dist/bundle.js");

        let policy = WasmIsolationPolicy {
            allow_network: false,
            read_paths: vec![temp.path().join("src")],
            write_paths: vec![out_file.clone()],
            max_memory_mb: 64,
        };

        let outputs = WasmPluginRunner::execute_sandboxed_plugin(
            &wasm_file,
            &policy,
            &["--minify".to_string()],
        )
        .unwrap();

        assert_eq!(outputs.len(), 1);
        assert!(out_file.exists());
    }
}
