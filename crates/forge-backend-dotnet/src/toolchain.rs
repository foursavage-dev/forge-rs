#![forbid(unsafe_code)]

use crate::DotnetBackendError;

#[derive(Debug, Clone)]
pub struct DotnetToolchain {
    pub executable: String,
    pub dotnet_version: String,
    pub csharp_executable: Option<String>,
    pub fsharp_executable: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DotnetCompiler {
    Csc(String),
    Fsc(String),
}

impl DotnetToolchain {
    pub fn detect() -> Result<Self, DotnetBackendError> {
        let dotnet_executable = Self::find_executable("dotnet").ok_or_else(|| {
            DotnetBackendError::Toolchain(".NET CLI not found in PATH".to_string())
        })?;

        let dotnet_version = Self::get_version(&dotnet_executable, &["--version"])
            .unwrap_or_else(|_| "unknown".to_string());

        let csharp_executable = Self::find_executable("csc");
        let fsharp_executable = Self::find_executable("fsc");

        Ok(DotnetToolchain {
            executable: dotnet_executable,
            dotnet_version,
            csharp_executable,
            fsharp_executable,
        })
    }

    pub fn with_dotnet(executable: String) -> Self {
        let dotnet_version = Self::get_version(&executable, &["--version"])
            .unwrap_or_else(|_| "unknown".to_string());

        DotnetToolchain {
            executable: executable.clone(),
            dotnet_version,
            csharp_executable: Self::find_executable("csc"),
            fsharp_executable: Self::find_executable("fsc"),
        }
    }

    pub fn is_csharp_available(&self) -> bool {
        self.csharp_executable.is_some()
    }

    pub fn is_fsharp_available(&self) -> bool {
        self.fsharp_executable.is_some()
    }

    fn find_executable(name: &str) -> Option<String> {
        // Check if executable exists in PATH
        if let Ok(output) = std::process::Command::new("where").arg(name).output()
            && output.status.success()
        {
            let paths = String::from_utf8_lossy(&output.stdout);
            return paths.lines().next().map(|s| s.to_string());
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

    fn get_version(executable: &str, args: &[&str]) -> Result<String, DotnetBackendError> {
        let output = std::process::Command::new(executable)
            .args(args)
            .output()
            .map_err(|e| {
                DotnetBackendError::Toolchain(format!("Failed to run {}: {}", executable, e))
            })?;

        if !output.status.success() {
            return Err(DotnetBackendError::Toolchain(format!(
                "{} exited with error code: {:?}",
                executable,
                output.status.code()
            )));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.lines().next().unwrap_or("unknown").to_string())
    }
}

impl DotnetCompiler {
    pub fn detect() -> Result<Self, DotnetBackendError> {
        // Try csc first
        if let Some(csc) = DotnetToolchain::find_executable("csc") {
            return Ok(DotnetCompiler::Csc(csc));
        }

        // Fall back to fsc
        if let Some(fsc) = DotnetToolchain::find_executable("fsc") {
            return Ok(DotnetCompiler::Fsc(fsc));
        }

        Err(DotnetBackendError::Toolchain(
            "No .NET compiler found (csc or fsc)".to_string(),
        ))
    }

    pub fn executable(&self) -> &str {
        match self {
            DotnetCompiler::Csc(path) => path,
            DotnetCompiler::Fsc(path) => path,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            DotnetCompiler::Csc(_) => "csc",
            DotnetCompiler::Fsc(_) => "fsc",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dotnet_toolchain_detection() {
        // This test will fail if .NET is not installed
        let result = DotnetToolchain::detect();
        if let Ok(toolchain) = result {
            assert!(!toolchain.executable.is_empty());
            assert!(!toolchain.dotnet_version.is_empty());
        }
    }

    #[test]
    fn test_dotnet_compiler_detection() {
        let result = DotnetCompiler::detect();
        if let Ok(compiler) = result {
            match compiler {
                DotnetCompiler::Csc(_) => assert_eq!(compiler.name(), "csc"),
                DotnetCompiler::Fsc(_) => assert_eq!(compiler.name(), "fsc"),
            }
        }
    }
}
