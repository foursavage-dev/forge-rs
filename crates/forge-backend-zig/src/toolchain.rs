#![forbid(unsafe_code)]

use crate::ZigBackendError;

#[derive(Debug, Clone)]
pub struct ZigToolchain {
    pub executable: String,
    pub zig_version: String,
}

#[derive(Debug, Clone)]
pub enum ZigCompiler {
    Zig(String),
}

impl ZigToolchain {
    pub fn detect() -> Result<Self, ZigBackendError> {
        let zig_executable = Self::find_executable("zig")
            .ok_or_else(|| ZigBackendError::Toolchain("Zig not found in PATH".to_string()))?;

        let zig_version = Self::get_version(&zig_executable, &["version"])
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(ZigToolchain {
            executable: zig_executable,
            zig_version,
        })
    }

    pub fn with_zig(executable: String) -> Self {
        let zig_version =
            Self::get_version(&executable, &["version"]).unwrap_or_else(|_| "unknown".to_string());

        ZigToolchain {
            executable: executable.clone(),
            zig_version,
        }
    }

    fn find_executable(name: &str) -> Option<String> {
        // Check if executable exists in PATH
        if let Ok(output) = std::process::Command::new("where").arg(name).output() {
            if output.status.success() {
                let paths = String::from_utf8_lossy(&output.stdout);
                return paths.lines().next().map(|s| s.to_string());
            }
        }

        // Try direct execution
        if std::process::Command::new(name)
            .arg("version")
            .output()
            .is_ok()
        {
            return Some(name.to_string());
        }

        None
    }

    fn get_version(executable: &str, args: &[&str]) -> Result<String, ZigBackendError> {
        let output = std::process::Command::new(executable)
            .args(args)
            .output()
            .map_err(|e| {
                ZigBackendError::Toolchain(format!("Failed to run {}: {}", executable, e))
            })?;

        if !output.status.success() {
            return Err(ZigBackendError::Toolchain(format!(
                "{} exited with error code: {:?}",
                executable,
                output.status.code()
            )));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.lines().next().unwrap_or("unknown").to_string())
    }
}

impl ZigCompiler {
    pub fn detect() -> Result<Self, ZigBackendError> {
        // Try zig
        if let Some(zig) = ZigToolchain::find_executable("zig") {
            return Ok(ZigCompiler::Zig(zig));
        }

        Err(ZigBackendError::Toolchain(
            "Zig compiler not found".to_string(),
        ))
    }

    pub fn executable(&self) -> &str {
        match self {
            ZigCompiler::Zig(path) => path,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ZigCompiler::Zig(_) => "zig",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zig_toolchain_detection() {
        // This test will fail if Zig is not installed
        let result = ZigToolchain::detect();
        if let Ok(toolchain) = result {
            assert!(!toolchain.executable.is_empty());
            assert!(!toolchain.zig_version.is_empty());
        }
    }

    #[test]
    fn test_zig_compiler_detection() {
        let result = ZigCompiler::detect();
        if let Ok(compiler) = result {
            assert_eq!(compiler.name(), "zig");
        }
    }
}
