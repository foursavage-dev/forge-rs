#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ZigTarget {
    Native,
    X86_64Linux,
    X86_64Windows,
    X86_64MacOS,
    Aarch64Linux,
    Aarch64MacOS,
    Wasm32,
    Custom(String),
}

impl ZigTarget {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "native" => ZigTarget::Native,
            "x86_64-linux" | "x86_64-linux-gnu" => ZigTarget::X86_64Linux,
            "x86_64-windows" | "x86_64-windows-gnu" => ZigTarget::X86_64Windows,
            "x86_64-macos" | "x86_64-macos-gnu" => ZigTarget::X86_64MacOS,
            "aarch64-linux" | "aarch64-linux-gnu" => ZigTarget::Aarch64Linux,
            "aarch64-macos" | "aarch64-macos-gnu" => ZigTarget::Aarch64MacOS,
            "wasm32" | "wasm32-wasi" => ZigTarget::Wasm32,
            custom => ZigTarget::Custom(custom.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ZigTarget::Native => "native",
            ZigTarget::X86_64Linux => "x86_64-linux-gnu",
            ZigTarget::X86_64Windows => "x86_64-windows-gnu",
            ZigTarget::X86_64MacOS => "x86_64-macos-gnu",
            ZigTarget::Aarch64Linux => "aarch64-linux-gnu",
            ZigTarget::Aarch64MacOS => "aarch64-macos-gnu",
            ZigTarget::Wasm32 => "wasm32-wasi",
            ZigTarget::Custom(s) => s,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigProjectConfig {
    pub project_name: String,
    pub target: ZigTarget,
    pub release: bool,
    pub run_tests: bool,
}

impl ZigProjectConfig {
    pub fn from_build_zig(project_dir: &Path) -> Result<Self, String> {
        let build_zig_path = project_dir.join("build.zig");
        if !build_zig_path.exists() {
            return Err("build.zig not found".to_string());
        }

        let content = std::fs::read_to_string(&build_zig_path)
            .map_err(|e| format!("Failed to read build.zig: {}", e))?;

        // Extract project name from build.zig
        let project_name = Self::extract_project_name(&content)
            .unwrap_or_else(|| "zig_project".to_string());

        // Detect target (default to native)
        let target = Self::detect_target(&content);

        Ok(ZigProjectConfig {
            project_name,
            target,
            release: false,
            run_tests: true,
        })
    }

    pub fn detect(project_dir: &Path) -> Result<Self, String> {
        if project_dir.join("build.zig").exists() {
            return Self::from_build_zig(project_dir);
        }

        Err("No Zig project files found (build.zig)".to_string())
    }

    fn extract_project_name(content: &str) -> Option<String> {
        // Try to extract project name from build.zig
        // Look for patterns like const name = "project_name"
        let patterns: [&str; 2] = [
            r#"const\s+\w+\s*=\s*"([^"]+)""#,
            r#"const\s+\w+\s*=\s*'([^']+)'"#,
        ];

        for pattern in patterns {
            if let Some(name) = Self::extract_with_pattern(pattern, content) {
                return Some(name);
            }
        }

        None
    }

    fn extract_with_pattern(pattern: &str, text: &str) -> Option<String> {
        // Simple pattern matching - in production use regex crate
        let pattern_parts: Vec<&str> = pattern.split("\"").collect();
        if pattern_parts.len() >= 2 {
            if let Some(start) = text.find(pattern_parts[0]) {
                let after_start = &text[start + pattern_parts[0].len()..];
                if let Some(end) = after_start.find(pattern_parts[1]) {
                    return Some(after_start[..end].to_string());
                }
            }
        }
        None
    }

    fn detect_target(content: &str) -> ZigTarget {
        // Try to detect target from build.zig
        if content.contains("x86_64-linux") {
            return ZigTarget::X86_64Linux;
        }
        if content.contains("x86_64-windows") {
            return ZigTarget::X86_64Windows;
        }
        if content.contains("x86_64-macos") {
            return ZigTarget::X86_64MacOS;
        }
        if content.contains("aarch64-linux") {
            return ZigTarget::Aarch64Linux;
        }
        if content.contains("aarch64-macos") {
            return ZigTarget::Aarch64MacOS;
        }
        if content.contains("wasm32") {
            return ZigTarget::Wasm32;
        }
        
        // Default to native
        ZigTarget::Native
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_build_zig_config_detection() {
        let temp = tempdir().unwrap();
        let build_zig_content = r#"
const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions({});
    const optimize = b.standardOptimizeOption(.{});
    
    const exe = b.addExecutable(.{
        .name = "test_app",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });
    
    b.installArtifact(exe);
}
"#;
        std::fs::write(temp.path().join("build.zig"), build_zig_content).unwrap();

        let config = ZigProjectConfig::from_build_zig(temp.path()).unwrap();
        assert_eq!(config.project_name, "test_app");
        assert_eq!(config.target, ZigTarget::Native);
    }

    #[test]
    fn test_target_parsing() {
        assert_eq!(ZigTarget::from_str("native"), ZigTarget::Native);
        assert_eq!(ZigTarget::from_str("x86_64-linux"), ZigTarget::X86_64Linux);
        assert_eq!(ZigTarget::from_str("wasm32"), ZigTarget::Wasm32);
    }
}
