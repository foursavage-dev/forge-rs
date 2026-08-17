#![forbid(unsafe_code)]

use crate::JavaBackendError;

#[derive(Debug, Clone)]
pub struct JavaToolchain {
    pub java_executable: String,
    pub java_version: String,
    pub javac_executable: Option<String>,
    pub kotlin_executable: Option<String>,
    pub kotlin_version: Option<String>,
    pub maven_executable: Option<String>,
    pub gradle_executable: Option<String>,
}

#[derive(Debug, Clone)]
pub enum JavaCompiler {
    Javac(String),
    Kotlinc(String),
}

impl JavaToolchain {
    pub fn detect() -> Result<Self, JavaBackendError> {
        let java_executable = Self::find_executable("java")
            .ok_or_else(|| JavaBackendError::Toolchain("Java not found in PATH".to_string()))?;

        let java_version = Self::get_version(&java_executable, &["-version"])
            .unwrap_or_else(|_| "unknown".to_string());

        let javac_executable = Self::find_executable("javac");
        let kotlin_executable = Self::find_executable("kotlinc");
        let kotlin_version = kotlin_executable
            .as_ref()
            .and_then(|k| Self::get_version(k, &["-version"]).ok());

        let maven_executable = Self::find_executable("mvn");
        let gradle_executable = Self::find_executable("gradle");

        Ok(JavaToolchain {
            java_executable,
            java_version,
            javac_executable,
            kotlin_executable,
            kotlin_version,
            maven_executable,
            gradle_executable,
        })
    }

    pub fn with_java(executable: String) -> Self {
        let java_version =
            Self::get_version(&executable, &["-version"]).unwrap_or_else(|_| "unknown".to_string());

        JavaToolchain {
            java_executable: executable.clone(),
            java_version,
            javac_executable: Self::find_executable("javac"),
            kotlin_executable: Self::find_executable("kotlinc"),
            kotlin_version: None,
            maven_executable: Self::find_executable("mvn"),
            gradle_executable: Self::find_executable("gradle"),
        }
    }

    pub fn is_kotlin_available(&self) -> bool {
        self.kotlin_executable.is_some()
    }

    pub fn is_maven_available(&self) -> bool {
        self.maven_executable.is_some()
    }

    pub fn is_gradle_available(&self) -> bool {
        self.gradle_executable.is_some()
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

    fn get_version(executable: &str, args: &[&str]) -> Result<String, JavaBackendError> {
        let output = std::process::Command::new(executable)
            .args(args)
            .output()
            .map_err(|e| {
                JavaBackendError::Toolchain(format!("Failed to run {}: {}", executable, e))
            })?;

        if !output.status.success() {
            return Err(JavaBackendError::Toolchain(format!(
                "{} exited with error code: {:?}",
                executable,
                output.status.code()
            )));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.lines().next().unwrap_or("unknown").to_string())
    }
}

impl JavaCompiler {
    pub fn detect() -> Result<Self, JavaBackendError> {
        // Try kotlinc first
        if let Some(kotlinc) = JavaToolchain::find_executable("kotlinc") {
            return Ok(JavaCompiler::Kotlinc(kotlinc));
        }

        // Fall back to javac
        if let Some(javac) = JavaToolchain::find_executable("javac") {
            return Ok(JavaCompiler::Javac(javac));
        }

        Err(JavaBackendError::Toolchain(
            "No Java compiler found (javac or kotlinc)".to_string(),
        ))
    }

    pub fn executable(&self) -> &str {
        match self {
            JavaCompiler::Javac(path) => path,
            JavaCompiler::Kotlinc(path) => path,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            JavaCompiler::Javac(_) => "javac",
            JavaCompiler::Kotlinc(_) => "kotlinc",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_toolchain_detection() {
        // This test will fail if Java is not installed
        let result = JavaToolchain::detect();
        if let Ok(toolchain) = result {
            assert!(!toolchain.java_executable.is_empty());
            assert!(!toolchain.java_version.is_empty());
        }
    }

    #[test]
    fn test_java_compiler_detection() {
        let result = JavaCompiler::detect();
        if let Ok(compiler) = result {
            match compiler {
                JavaCompiler::Javac(_) => assert_eq!(compiler.name(), "javac"),
                JavaCompiler::Kotlinc(_) => assert_eq!(compiler.name(), "kotlinc"),
            }
        }
    }
}
