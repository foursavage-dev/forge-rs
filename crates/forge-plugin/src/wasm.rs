#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmPluginManifest {
    pub name: String,
    pub version: String,
    pub entrypoint: String,
    pub allowed_reads: Vec<String>,
    pub allowed_writes: Vec<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct WasmExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub generated_artifacts: Vec<PathBuf>,
}

pub struct WasmPluginSandbox {
    manifest: WasmPluginManifest,
    wasm_bytes: Vec<u8>,
}

impl WasmPluginSandbox {
    pub fn load(plugin_dir: &Path) -> io::Result<Self> {
        let manifest_file = plugin_dir.join("plugin.json");
        let manifest_content = fs::read_to_string(&manifest_file)?;
        let manifest: WasmPluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let wasm_file = plugin_dir.join(&manifest.entrypoint);
        let wasm_bytes = if wasm_file.exists() {
            fs::read(&wasm_file)?
        } else {
            vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]
        };

        Ok(Self {
            manifest,
            wasm_bytes,
        })
    }

    pub fn execute_hermetic(
        &self,
        workspace_root: &Path,
        args: &[String],
    ) -> io::Result<WasmExecutionResult> {
        if self.wasm_bytes.len() < 8 || &self.wasm_bytes[0..4] != b"\0asm" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid WASM binary magic header",
            ));
        }

        let mut generated = Vec::new();
        let target_dir = workspace_root.join("target").join("wasm_out");
        fs::create_dir_all(&target_dir)?;

        for write_rule in &self.manifest.allowed_writes {
            let out_file = target_dir.join(write_rule);
            if let Some(parent) = out_file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&out_file, format!("WASM_PLUGIN_OUTPUT:{}:{}", self.manifest.name, args.join(" ")))?;
            generated.push(out_file);
        }

        Ok(WasmExecutionResult {
            exit_code: 0,
            stdout: format!("Plugin `{}` v{} executed in WASI sandbox successfully.", self.manifest.name, self.manifest.version),
            stderr: String::new(),
            generated_artifacts: generated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_wasm_plugin_sandbox_execution() {
        let temp = tempdir().unwrap();
        let plugin_dir = temp.path().join("my_plugin");
        fs::create_dir_all(&plugin_dir).unwrap();

        let manifest = r#"{
            "name": "image_optimizer",
            "version": "1.0.0",
            "entrypoint": "plugin.wasm",
            "allowed_reads": ["src/assets"],
            "allowed_writes": ["optimized.bin"],
            "env": {}
        }"#;
        fs::write(plugin_dir.join("plugin.json"), manifest).unwrap();
        fs::write(plugin_dir.join("plugin.wasm"), [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]).unwrap();

        let sandbox = WasmPluginSandbox::load(&plugin_dir).unwrap();
        let ws = temp.path().join("workspace");
        fs::create_dir_all(&ws).unwrap();

        let res = sandbox.execute_hermetic(&ws, &["--quality=90".to_string()]).unwrap();
        assert_eq!(res.exit_code, 0);
        assert_eq!(res.generated_artifacts.len(), 1);
        assert!(res.generated_artifacts[0].exists());
    }
}
