#![forbid(unsafe_code)]

use crate::SwiftBackendError;

#[derive(Debug, Clone)]
pub struct SwiftToolchain {
    pub executable: String,
    pub swift_version: String,
    pub clang_executable: Option<String>,
    pub clang_version: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SwiftCompiler {
    Swift(String),
    Clang(String),
}

impl SwiftToolchain {
    pub fn detect() -> Result<Self, SwiftBackendError> {
        let swift_executable = Self::find_executable("swift")
            .ok_or_else(|| SwiftBackendError::Toolchain("Swift not found in PATH".to_string()))?;
        
        let swift_version = Self::get_version(&swift_executable, &["--version"])
            .unwrap_or_else(|_| "unknown".to_string());

        let clang_executable = Self::find_executable("clang");
        let clang_version = clang_executable.as_ref()
            .and_then(|c| Self::get_version(c, &["--version"]).ok());

        Ok(SwiftToolchain {
            executable: swift_executable,
            swift_version,
            clang_executable,
            clang_version,
        })
    }

    pub fn with_swift(executable: String) -> Self {
        let swift_version = Self::get_version(&executable, &["--version"])
            .unwrap_or_else(|_| "unknown".to_string());
        
        SwiftToolchain {
            executable: executable.clone(),
            swift_version,
            clang_executable: Self::find_executable("clang"),
            clang_version: None,
        }
    }

    pub fn is_clang_available(&self) -> bool {
        self.clang_executable.is_some()
    }

    fn find_executable(name: &str) -> Option<String> {
        // Check if executable exists in PATH
        if let Ok(output) = std::process::Command::new("where")
            .arg(name)
            .output()
        {
            if output.status.success() {
                let paths = String::from_utf8_lossy(&output.stdout);
                return paths.lines().next().map(|s| s.to_string());
            }
        }
        
        // Try direct execution
        if std::process::Command::new(name)
            .arg("--version")
            .output()
            .is_ok()
            || std::process::Command::new(name)
            .arg("-version")
            .output()
            .is_ok()
        {
            return Some(name.to_string());
        }

        None
    }

    fn get_version(executable: &str, args: &[&str]) -> Result<String, SwiftBackendError> {
        let output = std::process::Command::new(executable)
            .args(args)
            .output()
            .map_err(|e| SwiftBackendError::Toolchain(format!("Failed to run {}: {}", executable, e)))?;

        if !output.status.success() {
            return Err(SwiftBackendError::Toolchain(format!(
                "{} exited with error code: {:?}",
                executable, output.status.code()
            )));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.lines().next().unwrap_or("unknown").to_string())
    }
}

impl SwiftCompiler {
    pub fn detect() -> Result<Self, SwiftBackendError> {
        // Try swift first
        if let Some(swift) = SwiftToolchain::find_executable("swift") {
            return Ok(SwiftCompiler::Swift(swift));
        }

        // Fall back to clang
        if let Some(clang) = SwiftToolchain::find_executable("clang") {
            return Ok(SwiftCompiler::Clang(clang));
        }

        Err(SwiftBackendError::Toolchain("No Swift compiler found (swift or clang)".to_string()))
    }

    pub fn executable(&self) -> &str {
        match self {
            SwiftCompiler::Swift(path) => path,
            SwiftCompiler::Clang(path) => path,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SwiftCompiler::Swift(_) => "swift",
            SwiftCompiler::Clang(_) => "clang",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swift_toolchain_detection() {
        // This test will fail if Swift is not installed
        let result = SwiftToolchain::detect();
        if let Ok(toolchain) = result {
            assert!(!toolchain.executable.is_empty());
            assert!(!toolchain.swift_version.is_empty());
        }
    }

    #[test]
    fn test_swift_compiler_detection() {
        let result = SwiftCompiler::detect();
        if let Ok(compiler) = result {
            match compiler {
                SwiftCompiler::Swift(_) => assert_eq!(compiler.name(), "swift"),
                SwiftCompiler::Clang(_) => assert_eq!(compiler.name(), "clang"),
            }
        }
    }
}
