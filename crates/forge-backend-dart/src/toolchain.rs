#![forbid(unsafe_code)]

use crate::DartBackendError;

#[derive(Debug, Clone)]
pub struct DartToolchain {
    pub dart_executable: Option<String>,
    pub dart_version: String,
    pub flutter_executable: Option<String>,
    pub flutter_version: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DartCompiler {
    Dart(String),
    Flutter(String),
}

impl DartToolchain {
    pub fn detect() -> Result<Self, DartBackendError> {
        let dart_executable = Self::find_executable("dart");
        let dart_version = dart_executable.as_ref()
            .and_then(|d| Self::get_version(d, &["--version"]).ok())
            .unwrap_or_else(|| "unknown".to_string());

        let flutter_executable = Self::find_executable("flutter");
        let flutter_version = flutter_executable.as_ref()
            .and_then(|f| Self::get_version(f, &["--version"]).ok());

        Ok(DartToolchain {
            dart_executable,
            dart_version,
            flutter_executable,
            flutter_version,
        })
    }

    pub fn with_dart(executable: String) -> Self {
        let dart_version = Self::get_version(&executable, &["--version"])
            .unwrap_or_else(|_| "unknown".to_string());
        
        DartToolchain {
            dart_executable: Some(executable.clone()),
            dart_version,
            flutter_executable: Self::find_executable("flutter"),
            flutter_version: None,
        }
    }

    pub fn is_flutter_available(&self) -> bool {
        self.flutter_executable.is_some()
    }

    pub fn is_dart_available(&self) -> bool {
        self.dart_executable.is_some()
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

    fn get_version(executable: &str, args: &[&str]) -> Result<String, DartBackendError> {
        let output = std::process::Command::new(executable)
            .args(args)
            .output()
            .map_err(|e| DartBackendError::Toolchain(format!("Failed to run {}: {}", executable, e)))?;

        if !output.status.success() {
            return Err(DartBackendError::Toolchain(format!(
                "{} exited with error code: {:?}",
                executable, output.status.code()
            )));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.lines().next().unwrap_or("unknown").to_string())
    }
}

impl DartCompiler {
    pub fn detect() -> Result<Self, DartBackendError> {
        // Try flutter first
        if let Some(flutter) = DartToolchain::find_executable("flutter") {
            return Ok(DartCompiler::Flutter(flutter));
        }

        // Fall back to dart
        if let Some(dart) = DartToolchain::find_executable("dart") {
            return Ok(DartCompiler::Dart(dart));
        }

        Err(DartBackendError::Toolchain("No Dart compiler found (dart or flutter)".to_string()))
    }

    pub fn executable(&self) -> &str {
        match self {
            DartCompiler::Dart(path) => path,
            DartCompiler::Flutter(path) => path,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            DartCompiler::Dart(_) => "dart",
            DartCompiler::Flutter(_) => "flutter",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dart_toolchain_detection() {
        // This test will fail if Dart is not installed
        let result = DartToolchain::detect();
        if let Ok(toolchain) = result {
            if toolchain.is_dart_available() {
                assert!(!toolchain.dart_version.is_empty());
            }
        }
    }

    #[test]
    fn test_dart_compiler_detection() {
        let result = DartCompiler::detect();
        if let Ok(compiler) = result {
            match compiler {
                DartCompiler::Dart(_) => assert_eq!(compiler.name(), "dart"),
                DartCompiler::Flutter(_) => assert_eq!(compiler.name(), "flutter"),
            }
        }
    }
}
