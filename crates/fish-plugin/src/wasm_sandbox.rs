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

pub struct WasmPluginSandbox {
    config: WasmPluginConfig,
    active: bool,
}

impl WasmPluginSandbox {
    pub fn new(config: WasmPluginConfig) -> Self {
        Self {
            config,
            active: true,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn config(&self) -> &WasmPluginConfig {
        &self.config
    }

    pub fn execute_function(&self, fn_name: &str, payload: &[u8]) -> Result<Vec<u8>, String> {
        if !self.active {
            return Err("Sandbox terminated".to_string());
        }
        let out = format!(
            "{{\"status\":\"success\",\"fn\":\"{}\",\"input_len\":{}}}",
            fn_name,
            payload.len()
        )
        .into_bytes();
        Ok(out)
    }

    pub fn terminate(&mut self) {
        self.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        sandbox.terminate();
        assert!(!sandbox.is_active());
        assert!(
            sandbox
                .execute_function("hook_pre_build", b"payload")
                .is_err()
        );
    }
}
