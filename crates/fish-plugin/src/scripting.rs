#![forbid(unsafe_code)]

//! Advanced plugin scripting system
//!
//! This module provides a more powerful plugin system that goes beyond
//! basic JSON configuration, allowing for custom build rules and logic.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Script-based plugin that can execute custom logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptPlugin {
    pub name: String,
    pub version: String,
    pub script_type: ScriptType,
    pub entry_point: PathBuf,
    pub dependencies: Vec<String>,
    pub capabilities: PluginCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptType {
    /// Simple shell script
    Shell,
    /// Python script
    Python,
    /// Node.js script
    Node,
    /// WASM module
    Wasm,
    /// Lua script
    Lua,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    pub can_build: bool,
    pub can_test: bool,
    pub can_clean: bool,
    pub can_graph: bool,
    pub supports_watch: bool,
}

impl ScriptPlugin {
    pub fn execute(&self, command: &str, args: &[String]) -> Result<PluginOutput, PluginError> {
        match self.script_type {
            ScriptType::Shell => self.execute_shell(command, args),
            ScriptType::Python => self.execute_python(command, args),
            ScriptType::Node => self.execute_node(command, args),
            ScriptType::Wasm => self.execute_wasm(command, args),
            ScriptType::Lua => self.execute_lua(command, args),
        }
    }

    fn execute_shell(&self, command: &str, args: &[String]) -> Result<PluginOutput, PluginError> {
        let mut cmd = Command::new(command);
        cmd.args(args);

        let output = cmd.output().map_err(|e| PluginError::Execution {
            command: command.to_string(),
            message: e.to_string(),
        })?;

        Ok(PluginOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        })
    }

    fn execute_python(&self, command: &str, args: &[String]) -> Result<PluginOutput, PluginError> {
        let mut cmd = Command::new("python3");
        cmd.arg(self.entry_point.clone()).arg(command).args(args);

        let output = cmd.output().map_err(|e| PluginError::Execution {
            command: "python3".to_string(),
            message: e.to_string(),
        })?;

        Ok(PluginOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        })
    }

    fn execute_node(&self, command: &str, args: &[String]) -> Result<PluginOutput, PluginError> {
        let mut cmd = Command::new("node");
        cmd.arg(self.entry_point.clone()).arg(command).args(args);

        let output = cmd.output().map_err(|e| PluginError::Execution {
            command: "node".to_string(),
            message: e.to_string(),
        })?;

        Ok(PluginOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        })
    }

    fn execute_wasm(&self, _command: &str, _args: &[String]) -> Result<PluginOutput, PluginError> {
        // WASM execution would require a WASM runtime
        // For now, return an error
        Err(PluginError::Unsupported(
            "WASM execution not yet implemented".to_string(),
        ))
    }

    fn execute_lua(&self, command: &str, args: &[String]) -> Result<PluginOutput, PluginError> {
        let mut cmd = Command::new("lua");
        cmd.arg(self.entry_point.clone()).arg(command).args(args);

        let output = cmd.output().map_err(|e| PluginError::Execution {
            command: "lua".to_string(),
            message: e.to_string(),
        })?;

        Ok(PluginOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PluginOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub enum PluginError {
    Execution { command: String, message: String },
    Unsupported(String),
    InvalidConfig(String),
    DependencyMissing(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::Execution { command, message } => {
                write!(f, "Plugin execution failed for '{}': {}", command, message)
            }
            PluginError::Unsupported(msg) => {
                write!(f, "Unsupported plugin feature: {}", msg)
            }
            PluginError::InvalidConfig(msg) => {
                write!(f, "Invalid plugin configuration: {}", msg)
            }
            PluginError::DependencyMissing(dep) => {
                write!(f, "Missing plugin dependency: {}", dep)
            }
        }
    }
}

impl std::error::Error for PluginError {}

/// Plugin manager for loading and managing script plugins
pub struct PluginManager {
    plugins: HashMap<String, ScriptPlugin>,
    plugin_dir: PathBuf,
}

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_dir,
        }
    }

    pub fn load_plugins(&mut self) -> Result<(), PluginError> {
        if !self.plugin_dir.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.plugin_dir).map_err(|e| {
            PluginError::InvalidConfig(format!("Cannot read plugin directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                PluginError::InvalidConfig(format!("Cannot read plugin entry: {}", e))
            })?;
            let path = entry.path();

            if path.is_dir()
                && let Ok(plugin) = self.load_plugin_from_dir(&path)
            {
                self.plugins.insert(plugin.name.clone(), plugin);
            }
        }

        Ok(())
    }

    fn load_plugin_from_dir(&self, dir: &Path) -> Result<ScriptPlugin, PluginError> {
        let config_path = dir.join("plugin.json");
        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| PluginError::InvalidConfig(format!("Cannot read plugin config: {}", e)))?;

        let plugin: ScriptPlugin = serde_json::from_str(&config_content).map_err(|e| {
            PluginError::InvalidConfig(format!("Cannot parse plugin config: {}", e))
        })?;

        // Validate dependencies
        for dep in &plugin.dependencies {
            if !self.check_dependency(dep) {
                return Err(PluginError::DependencyMissing(dep.clone()));
            }
        }

        Ok(plugin)
    }

    fn check_dependency(&self, dep: &str) -> bool {
        // Check if the dependency command exists
        Command::new(dep).arg("--version").output().is_ok()
    }

    pub fn get_plugin(&self, name: &str) -> Option<&ScriptPlugin> {
        self.plugins.get(name)
    }

    pub fn list_plugins(&self) -> Vec<&ScriptPlugin> {
        self.plugins.values().collect()
    }

    pub fn execute_plugin(
        &self,
        name: &str,
        command: &str,
        args: &[String],
    ) -> Result<PluginOutput, PluginError> {
        let plugin = self
            .get_plugin(name)
            .ok_or_else(|| PluginError::InvalidConfig(format!("Plugin '{}' not found", name)))?;

        plugin.execute(command, args)
    }
}

/// Example plugin manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub script_type: ScriptType,
    pub entry_point: String,
    pub dependencies: Vec<String>,
    pub capabilities: PluginCapabilities,
    pub build_rules: Vec<BuildRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRule {
    pub name: String,
    pub pattern: String,
    pub command: String,
    pub outputs: Vec<String>,
    pub dependencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plugin_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PluginManager::new(temp_dir.path().to_path_buf());
        assert_eq!(manager.list_plugins().len(), 0);
    }

    #[test]
    fn test_plugin_execution() {
        let plugin = ScriptPlugin {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            script_type: ScriptType::Shell,
            entry_point: PathBuf::from("/bin/sh"),
            dependencies: vec![],
            capabilities: PluginCapabilities {
                can_build: true,
                can_test: false,
                can_clean: false,
                can_graph: false,
                supports_watch: false,
            },
        };

        // This would actually execute, so we just test the structure
        assert_eq!(plugin.name, "test");
    }
}
