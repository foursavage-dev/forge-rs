#![forbid(unsafe_code)]

pub mod config;
pub mod fingerprint;
pub mod toolchain;

pub use config::DockerProjectConfig;
pub use fingerprint::DockerFingerprinter;
pub use toolchain::DockerToolchain;

use forge_core::BuildBackend;
use forge_executor::CommandSpec;
use forge_graph::BuildGraph;
use std::fmt::Debug;

type DockerStages = Vec<(String, Vec<CommandSpec>)>;

#[derive(Debug)]
pub struct DockerBackend {
    config: DockerProjectConfig,
    toolchain: DockerToolchain,
    fingerprinter: DockerFingerprinter,
}

impl DockerBackend {
    pub fn new(config: DockerProjectConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let toolchain = DockerToolchain::detect()?;
        let fingerprinter = DockerFingerprinter::new(config.clone());
        
        Ok(Self {
            config,
            toolchain,
            fingerprinter,
        })
    }

    pub fn with_toolchain(config: DockerProjectConfig, toolchain: DockerToolchain) -> Self {
        let fingerprinter = DockerFingerprinter::new(config.clone());
        Self {
            config,
            toolchain,
            fingerprinter,
        }
    }
    
    pub fn detect_config(project_dir: &std::path::Path) -> Option<DockerProjectConfig> {
        let dockerfile = project_dir.join("Dockerfile");
        if dockerfile.exists() {
            return Some(DockerProjectConfig {
                dockerfile_path: Some(dockerfile),
                context_path: project_dir.to_path_buf(),
                build_args: std::collections::HashMap::new(),
                target: None,
                cache_from: Vec::new(),
                cache_to: Vec::new(),
            });
        }
        
        let compose_file = project_dir.join("docker-compose.yml");
        if compose_file.exists() {
            return Some(DockerProjectConfig {
                dockerfile_path: None,
                context_path: project_dir.to_path_buf(),
                build_args: std::collections::HashMap::new(),
                target: None,
                cache_from: Vec::new(),
                cache_to: Vec::new(),
            });
        }
        
        None
    }
    
    pub fn build_task_graph(&self) -> Result<BuildGraph<forge_executor::Task>, Box<dyn std::error::Error>> {
        let mut graph = BuildGraph::new();
        let fingerprint = self.fingerprinter.compute()?;
        let stages = self.parse_dockerfile()?;
        let stages_count = stages.len();
        
        for (stage_name, commands) in stages {
            let task_id = format!("docker:build:{}", stage_name);
            
            for cmd in commands {
                let mut task = forge_executor::Task::new(
                    task_id.clone(),
                    format!("Build Docker stage: {}", stage_name),
                    cmd,
                );
                
                if let Some(dockerfile) = &self.config.dockerfile_path {
                    task = task.with_cache(forge_executor::CacheEntry {
                        key: format!("docker:{}", dockerfile.display()),
                        fingerprint: fingerprint.clone(),
                    });
                }
                
                if stage_name == "final" || stages_count == 1 {
                    task = task.with_artifacts(vec![std::path::PathBuf::from(format!("{}.tar", stage_name))]);
                }
                
                let _node_id = graph.add_node(task);
            }
        }
        
        Ok(graph)
    }
    
    pub fn validate_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dockerfile) = &self.config.dockerfile_path {
            if !dockerfile.exists() {
                return Err(format!("Dockerfile not found: {}", dockerfile.display()).into());
            }
        }
        Ok(())
    }
}

impl BuildBackend for DockerBackend {
    fn name(&self) -> &'static str {
        "docker"
    }
}

impl DockerBackend {
    fn parse_dockerfile(&self) -> Result<DockerStages, Box<dyn std::error::Error>> {
        let dockerfile = self.config.dockerfile_path.as_ref()
            .ok_or("No Dockerfile specified")?;
        
        let content = std::fs::read_to_string(dockerfile)?;
        let mut stages = Vec::new();
        let mut current_stage = "default".to_string();
        let mut current_commands = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some(stage) = line.strip_prefix("FROM ") {
                if let Some(as_stage) = stage.split(" AS ").nth(1) {
                    if !current_commands.is_empty() {
                        stages.push((current_stage.clone(), current_commands.clone()));
                        current_commands.clear();
                    }
                    current_stage = as_stage.trim().to_string();
                }
            } else if line.starts_with("RUN ") || line.starts_with("COPY ") || line.starts_with("ADD ") {
                let docker_cmd = format!("build -t stage:{} {}", current_stage, self.config.context_path.display());
                let mut cmd = CommandSpec::new(self.toolchain.docker_path.to_string_lossy().to_string());
                for arg in docker_cmd.split_whitespace() {
                    cmd = cmd.arg(arg);
                }
                current_commands.push(cmd);
            }
        }
        
        if !current_commands.is_empty() {
            stages.push((current_stage, current_commands));
        }
        
        if stages.is_empty() {
            let docker_cmd = format!("build -t forge:latest {}", self.config.context_path.display());
            let mut cmd = CommandSpec::new(self.toolchain.docker_path.to_string_lossy().to_string());
            for arg in docker_cmd.split_whitespace() {
                cmd = cmd.arg(arg);
            }
            stages.push(("default".to_string(), vec![cmd]));
        }
        
        Ok(stages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_docker_backend_creation() {
        let config = DockerProjectConfig {
            dockerfile_path: Some(std::path::PathBuf::from("Dockerfile")),
            context_path: std::path::PathBuf::from("."),
            build_args: std::collections::HashMap::new(),
            target: None,
            cache_from: Vec::new(),
            cache_to: Vec::new(),
        };

        let toolchain = DockerToolchain {
            docker_path: std::path::PathBuf::from("docker"),
            is_podman: false,
            version: "24.0.0".to_string(),
        };
        
        let backend = DockerBackend::with_toolchain(config, toolchain);
        assert_eq!(backend.config.context_path, std::path::PathBuf::from("."));
    }
}
