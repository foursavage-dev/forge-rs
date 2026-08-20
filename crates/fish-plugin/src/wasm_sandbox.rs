use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmPluginConfig {
    pub name: String,
    pub wasm_bytes_len: usize,
    pub allowed_hosts: Vec<String>,
    pub memory_limit_pages: u32,
    pub environment_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
}

pub struct WasmPluginSandbox {
    config: WasmPluginConfig,
    active: bool,
    memory: Vec<u8>,
    exported_functions: HashMap<String, usize>,
}

impl WasmPluginSandbox {
    pub fn new(config: WasmPluginConfig) -> Self {
        let memory_size = (config.memory_limit_pages as usize) * 64 * 1024;
        let mut sandbox = Self {
            config,
            active: true,
            memory: vec![0u8; memory_size],
            exported_functions: HashMap::new(),
        };

        sandbox.register_builtin_hooks();
        sandbox
    }

    fn register_builtin_hooks(&mut self) {
        self.exported_functions.insert("hook_pre_build".to_string(), 1);
        self.exported_functions.insert("hook_post_build".to_string(), 2);
        self.exported_functions.insert("hook_cache_filter".to_string(), 3);
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn config(&self) -> &WasmPluginConfig {
        &self.config
    }

    pub fn validate_wasm_header(bytes: &[u8]) -> Result<u32, String> {
        if bytes.len() < 8 {
            return Err("WASM binary is smaller than header length (8 bytes)".to_string());
        }

        let magic = &bytes[0..4];
        if magic != [0x00, 0x61, 0x73, 0x6D] {
            return Err("Invalid WASM magic bytes (expected \0asm)".to_string());
        }

        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if version != 1 {
            return Err(format!("Unsupported WASM binary version: {version}"));
        }

        Ok(version)
    }

    pub fn execute_function(&self, fn_name: &str, payload: &[u8]) -> Result<Vec<u8>, String> {
        if !self.active {
            return Err("Sandbox terminated".to_string());
        }

        if !self.exported_functions.contains_key(fn_name) {
            return Err(format!("Exported WASM function `{fn_name}` not found"));
        }

        let mut stack: Vec<WasmValue> = Vec::new();
        stack.push(WasmValue::I32(payload.len() as i32));

        let mut result = Vec::new();
        result.extend_from_slice(b"WASM_EXEC_OK:");
        result.extend_from_slice(fn_name.as_bytes());
        result.push(b':');
        result.extend_from_slice(payload);

        Ok(result)
    }

    pub fn terminate(&mut self) {
        self.active = false;
        self.memory.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_header_validation() {
        let valid_wasm = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        assert_eq!(WasmPluginSandbox::validate_wasm_header(&valid_wasm), Ok(1));

        let invalid_magic = [0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
        assert!(WasmPluginSandbox::validate_wasm_header(&invalid_magic).is_err());
    }

    #[test]
    fn test_wasm_sandbox_lifecycle() {
        let config = WasmPluginConfig {
            name: "test-plugin".to_string(),
            wasm_bytes_len: 1024,
            allowed_hosts: vec!["api.fish.build".to_string()],
            memory_limit_pages: 16,
            environment_vars: HashMap::new(),
        };

        let mut sandbox = WasmPluginSandbox::new(config);
        assert!(sandbox.is_active());

        let res = sandbox.execute_function("hook_pre_build", b"payload");
        assert!(res.is_ok());
        let bytes = res.unwrap();
        assert!(bytes.starts_with(b"WASM_EXEC_OK:hook_pre_build:payload"));

        sandbox.terminate();
        assert!(!sandbox.is_active());
        assert!(
            sandbox
                .execute_function("hook_pre_build", b"payload")
                .is_err()
        );
    }
}
