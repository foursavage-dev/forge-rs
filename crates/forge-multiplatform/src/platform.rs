// Platform definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    ARM64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Target {
    pub platform: Platform,
    pub architecture: Architecture,
}

impl Target {
    pub fn new(platform: Platform, architecture: Architecture) -> Self {
        Self { platform, architecture }
    }

    pub fn to_rust_target(&self) -> String {
        match (self.platform, self.architecture) {
            (Platform::Linux, Architecture::X86_64) => "x86_64-unknown-linux-gnu".to_string(),
            (Platform::Linux, Architecture::ARM64) => "aarch64-unknown-linux-gnu".to_string(),
            (Platform::MacOS, Architecture::X86_64) => "x86_64-apple-darwin".to_string(),
            (Platform::MacOS, Architecture::ARM64) => "aarch64-apple-darwin".to_string(),
            (Platform::Windows, Architecture::X86_64) => "x86_64-pc-windows-msvc".to_string(),
            (Platform::Windows, Architecture::ARM64) => "aarch64-pc-windows-msvc".to_string(),
        }
    }
}
