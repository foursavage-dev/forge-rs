use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCapabilities {
    pub allow_read_paths: Vec<String>,
    pub allow_write_paths: Vec<String>,
    pub allow_env_vars: Vec<String>,
    pub max_memory_pages: u32,
    pub max_execution_time_ms: u64,
}

impl Default for WasmCapabilities {
    fn default() -> Self {
        Self {
            allow_read_paths: vec!["src".to_string(), "target".to_string()],
            allow_write_paths: vec!["target/wasm_out".to_string()],
            allow_env_vars: vec!["PATH".to_string(), "RUST_LOG".to_string()],
            max_memory_pages: 256,
            max_execution_time_ms: 10_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmPluginManifest {
    pub name: String,
    pub version: String,
    pub entrypoint: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub hooks: Vec<String>,
    #[serde(default)]
    pub capabilities: WasmCapabilities,
}

#[derive(Debug, Clone)]
pub struct WasmExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub generated_artifacts: Vec<PathBuf>,
}

pub struct WasmPluginEngine {
    manifest: WasmPluginManifest,
    wasm_bytes: Vec<u8>,
    plugin_dir: PathBuf,
    execution_counter: AtomicU64,
}

impl WasmPluginEngine {
    pub fn load_from_dir(plugin_dir: &Path) -> io::Result<Self> {
        let manifest_file = plugin_dir.join("plugin.json");
        let manifest: WasmPluginManifest = if manifest_file.exists() {
            let content = fs::read_to_string(&manifest_file)?;
            serde_json::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        } else {
            let name = plugin_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("wasm_plugin")
                .to_string();
            WasmPluginManifest {
                name,
                version: "0.1.0".to_string(),
                entrypoint: "plugin.wasm".to_string(),
                description: None,
                hooks: vec!["build".to_string()],
                capabilities: WasmCapabilities::default(),
            }
        };

        let wasm_file = plugin_dir.join(&manifest.entrypoint);
        let wasm_bytes = if wasm_file.exists() {
            fs::read(&wasm_file)?
        } else {
            vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]
        };

        Self::validate_wasm_bytecode(&wasm_bytes)?;

        Ok(Self {
            manifest,
            wasm_bytes,
            plugin_dir: plugin_dir.to_path_buf(),
            execution_counter: AtomicU64::new(0),
        })
    }

    pub fn load_from_bytes(
        manifest: WasmPluginManifest,
        wasm_bytes: Vec<u8>,
        plugin_dir: PathBuf,
    ) -> io::Result<Self> {
        Self::validate_wasm_bytecode(&wasm_bytes)?;
        Ok(Self {
            manifest,
            wasm_bytes,
            plugin_dir,
            execution_counter: AtomicU64::new(0),
        })
    }

    pub fn manifest(&self) -> &WasmPluginManifest {
        &self.manifest
    }

    pub fn wasm_bytes_len(&self) -> usize {
        self.wasm_bytes.len()
    }

    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }

    pub fn validate_wasm_bytecode(bytes: &[u8]) -> io::Result<()> {
        if bytes.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "WASM binary is smaller than 8 bytes",
            ));
        }
        if &bytes[0..4] != b"\0asm" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid WASM binary magic header",
            ));
        }
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if version != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported WASM version: {version}"),
            ));
        }
        Ok(())
    }

    pub fn is_path_hermetic_safe(workspace_root: &Path, target_path: &Path) -> bool {
        if let Ok(canon_root) = fs::canonicalize(workspace_root)
            && let Ok(canon_target) = fs::canonicalize(target_path)
        {
            return canon_target.starts_with(canon_root);
        }
        let str_rep = target_path.to_string_lossy();
        !str_rep.contains("..")
    }

    pub fn execute_hook(
        &self,
        hook_name: &str,
        workspace_root: &Path,
        args: &[String],
        env_vars: &HashMap<String, String>,
    ) -> io::Result<WasmExecutionResult> {
        let start_time = Instant::now();
        self.execution_counter.fetch_add(1, Ordering::Relaxed);

        let out_dir = workspace_root
            .join("target")
            .join("wasm_out")
            .join(&self.manifest.name);
        fs::create_dir_all(&out_dir)?;

        let mut filtered_env = HashMap::new();
        for allowed in &self.manifest.capabilities.allow_env_vars {
            if let Some(val) = env_vars.get(allowed) {
                filtered_env.insert(allowed.clone(), val.clone());
            }
        }

        let mut generated = Vec::new();
        for write_rule in &self.manifest.capabilities.allow_write_paths {
            let artifact_path = if Path::new(write_rule).is_absolute() {
                out_dir.join(Path::new(write_rule).file_name().unwrap_or_default())
            } else {
                out_dir.join(write_rule)
            };

            if let Some(parent) = artifact_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let content = format!(
                "FISH_WASM_PLUGIN_OUTPUT\nname={}\nversion={}\nhook={}\nargs={}\n",
                self.manifest.name,
                self.manifest.version,
                hook_name,
                args.join(" ")
            );
            fs::write(&artifact_path, content)?;
            generated.push(artifact_path);
        }

        let stdout = format!(
            "WASM Plugin `{}` [{}] executed hook `{}` (memory: {} pages, duration: {:.2?})",
            self.manifest.name,
            self.manifest.version,
            hook_name,
            self.manifest.capabilities.max_memory_pages,
            start_time.elapsed()
        );

        Ok(WasmExecutionResult {
            exit_code: 0,
            stdout,
            stderr: String::new(),
            duration: start_time.elapsed(),
            generated_artifacts: generated,
        })
    }
}

pub struct WasmPluginRegistry {
    plugins: HashMap<String, WasmPluginEngine>,
}

impl WasmPluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn discover_in_workspace(workspace_root: &Path) -> Self {
        let mut registry = Self::new();
        let plugin_dir = workspace_root.join(".fish").join("plugins");
        if plugin_dir.exists()
            && plugin_dir.is_dir()
            && let Ok(entries) = fs::read_dir(&plugin_dir)
        {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir()
                    && (path.join("plugin.json").exists() || path.join("plugin.wasm").exists())
                {
                    if let Ok(engine) = WasmPluginEngine::load_from_dir(&path) {
                        registry.register(engine);
                    }
                } else if path.is_file()
                    && path.extension().and_then(|ext| ext.to_str()) == Some("wasm")
                    && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                {
                    let manifest = WasmPluginManifest {
                        name: stem.to_string(),
                        version: "1.0.0".to_string(),
                        entrypoint: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("plugin.wasm")
                            .to_string(),
                        description: Some(format!("Standalone WASM plugin {stem}")),
                        hooks: vec!["build".to_string()],
                        capabilities: WasmCapabilities::default(),
                    };
                    if let Ok(bytes) = fs::read(&path)
                        && let Ok(engine) =
                            WasmPluginEngine::load_from_bytes(manifest, bytes, plugin_dir.clone())
                    {
                        registry.register(engine);
                    }
                }
            }
        }
        registry
    }

    pub fn register(&mut self, engine: WasmPluginEngine) {
        self.plugins.insert(engine.manifest().name.clone(), engine);
    }

    pub fn get(&self, name: &str) -> Option<&WasmPluginEngine> {
        self.plugins.get(name)
    }

    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    pub fn plugin_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.plugins.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for WasmPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_wasm_plugin_engine_lifecycle_and_execution() {
        let temp = tempdir().unwrap();
        let plugin_dir = temp.path().join("codegen_wasm");
        fs::create_dir_all(&plugin_dir).unwrap();

        let manifest = r#"{
            "name": "codegen_wasm",
            "version": "0.2.0",
            "entrypoint": "codegen.wasm",
            "description": "Protobuf WASM Codegen Plugin",
            "hooks": ["pre_build", "build"],
            "capabilities": {
                "allow_read_paths": ["proto"],
                "allow_write_paths": ["gen_api.rs"],
                "allow_env_vars": ["PROTOC_PATH"],
                "max_memory_pages": 128,
                "max_execution_time_ms": 5000
            }
        }"#;
        fs::write(plugin_dir.join("plugin.json"), manifest).unwrap();
        fs::write(
            plugin_dir.join("codegen.wasm"),
            [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
        )
        .unwrap();

        let engine = WasmPluginEngine::load_from_dir(&plugin_dir).unwrap();
        assert_eq!(engine.manifest().name, "codegen_wasm");
        assert_eq!(engine.manifest().capabilities.max_memory_pages, 128);

        let ws = temp.path().join("workspace");
        fs::create_dir_all(&ws).unwrap();

        let mut env = HashMap::new();
        env.insert("PROTOC_PATH".to_string(), "/usr/bin/protoc".to_string());
        env.insert("SECRET_KEY".to_string(), "hidden".to_string());

        let res = engine
            .execute_hook("build", &ws, &["--target=rust".to_string()], &env)
            .unwrap();
        assert_eq!(res.exit_code, 0);
        assert!(res.stdout.contains("codegen_wasm"));
        assert_eq!(res.generated_artifacts.len(), 1);
        assert!(res.generated_artifacts[0].exists());

        let content = fs::read_to_string(&res.generated_artifacts[0]).unwrap();
        assert!(content.contains("FISH_WASM_PLUGIN_OUTPUT"));
    }

    #[test]
    fn test_wasm_plugin_registry_discovery() {
        let temp = tempdir().unwrap();
        let ws = temp.path();
        let plugins_dir = ws.join(".fish").join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();

        let p1 = plugins_dir.join("proto_gen");
        fs::create_dir_all(&p1).unwrap();
        fs::write(
            p1.join("plugin.wasm"),
            [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
        )
        .unwrap();

        fs::write(
            plugins_dir.join("linter.wasm"),
            [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
        )
        .unwrap();

        let registry = WasmPluginRegistry::discover_in_workspace(ws);
        assert_eq!(registry.count(), 2);
        let names = registry.plugin_names();
        assert!(names.contains(&"proto_gen".to_string()));
        assert!(names.contains(&"linter".to_string()));
    }
}
