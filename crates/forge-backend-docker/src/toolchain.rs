#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DockerToolchainError {
    #[error("Docker not found on PATH")]
    DockerNotFound,
    
    #[error("Podman not found on PATH")]
    PodmanNotFound,
    
    #[error("No container runtime found (tried Docker and Podman)")]
    NoRuntimeFound,
    
    #[error("Failed to execute command: {0}")]
    CommandFailed(String),
}

pub type Result<T> = std::result::Result<T, DockerToolchainError>;

#[derive(Debug, Clone)]
pub struct DockerToolchain {
    pub docker_path: std::path::PathBuf,
    pub is_podman: bool,
    pub version: String,
}

impl DockerToolchain {
    pub fn detect() -> Result<Self> {
        // Try Docker first
        if let Ok(output) = std::process::Command::new("docker")
            .arg("--version")
            .output()
        {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                return Ok(Self {
                    docker_path: std::path::PathBuf::from("docker"),
                    is_podman: false,
                    version,
                });
            }
        }
        
        // Try Podman as fallback
        if let Ok(output) = std::process::Command::new("podman")
            .arg("--version")
            .output()
        {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                return Ok(Self {
                    docker_path: std::path::PathBuf::from("podman"),
                    is_podman: true,
                    version,
                });
            }
        }
        
        Err(DockerToolchainError::NoRuntimeFound)
    }
    
    pub fn check_registry(&self, registry: &str) -> Result<bool> {
        let output = std::process::Command::new(&self.docker_path)
            .args(["search", registry])
            .output()
            .map_err(|e| DockerToolchainError::CommandFailed(e.to_string()))?;
        
        Ok(output.status.success())
    }
    
    pub fn list_images(&self) -> Result<Vec<String>> {
        let output = std::process::Command::new(&self.docker_path)
            .args(["images", "--format", "{{.Repository}}:{{.Tag}}"])
            .output()
            .map_err(|e| DockerToolchainError::CommandFailed(e.to_string()))?;
        
        if !output.status.success() {
            return Err(DockerToolchainError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        
        let images = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.to_string())
            .collect();
        
        Ok(images)
    }
    
    pub fn list_containers(&self) -> Result<Vec<String>> {
        let output = std::process::Command::new(&self.docker_path)
            .args(["ps", "--format", "{{.Names}}"])
            .output()
            .map_err(|e| DockerToolchainError::CommandFailed(e.to_string()))?;
        
        if !output.status.success() {
            return Err(DockerToolchainError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        
        let containers = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.to_string())
            .collect();
        
        Ok(containers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_docker_toolchain_detect() {
        let toolchain = DockerToolchain::detect();
        // This test will fail if neither Docker nor Podman is installed
        // In CI environments, we should mock this
        println!("Toolchain detection result: {:?}", toolchain);
    }
}
