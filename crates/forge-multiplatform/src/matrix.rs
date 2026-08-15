// Test matrix configuration

use crate::platform::Target;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixConfig {
    pub platforms: Vec<Target>,
    pub include_rust_versions: Vec<String>,
    pub include_node_versions: Vec<String>,
}

impl Default for MatrixConfig {
    fn default() -> Self {
        Self {
            platforms: vec![
                Target::new(crate::platform::Platform::Linux, crate::platform::Architecture::X86_64),
                Target::new(crate::platform::Platform::MacOS, crate::platform::Architecture::ARM64),
                Target::new(crate::platform::Platform::Windows, crate::platform::Architecture::X86_64),
            ],
            include_rust_versions: vec!["stable".to_string(), "nightly".to_string()],
            include_node_versions: vec!["lts/*".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMatrix {
    pub targets: Vec<Target>,
    pub rust_versions: Vec<String>,
    pub node_versions: Vec<String>,
}
