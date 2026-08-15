use std::process::ExitCode;

use crate::args::{CiArgs, CiCommand};
use forge_ci_generator::{CIConfig, CIMatrix, CIJob, CIPlatform, GitHubActionsGenerator, GitLabCIGenerator};

pub fn run_ci(args: CiArgs) -> ExitCode {
    match args.command {
        CiCommand::Init { platform, cache, remote_cache } => {
            let ci_config = CIConfig {
                platform: match platform.as_str() {
                    "github" => CIPlatform::GitHubActions,
                    "gitlab" => CIPlatform::GitLabCI,
                    "both" => CIPlatform::Both,
                    _ => {
                        eprintln!("error: invalid platform '{}', expected 'github', 'gitlab', or 'both'", platform);
                        return ExitCode::FAILURE;
                    }
                },
                cache_enabled: cache,
                remote_cache_url: remote_cache.clone(),
                jobs_per_run: 4,
                timeout_minutes: 30,
            };
            
            // Create a sample CI matrix
            let mut matrix = CIMatrix::new();
            
            // Add sample jobs for demonstration
            matrix.add_job(CIJob {
                id: "build".to_string(),
                name: "Build".to_string(),
                backend: "rust".to_string(),
                commands: vec!["cargo build --release".to_string()],
                artifacts: vec!["target/release/my_app".to_string()],
                dependencies: vec![],
                cache_key: "build-cache".to_string(),
            });
            
            matrix.add_job(CIJob {
                id: "test".to_string(),
                name: "Test".to_string(),
                backend: "rust".to_string(),
                commands: vec!["cargo test".to_string()],
                artifacts: vec![],
                dependencies: vec!["build".to_string()],
                cache_key: "test-cache".to_string(),
            });
            
            matrix.cache_config.enabled = cache;
            matrix.cache_config.remote_url = remote_cache.clone();
            
            // Generate CI configuration files
            if ci_config.platform == CIPlatform::GitHubActions || 
               ci_config.platform == CIPlatform::Both {
                let generator = GitHubActionsGenerator::new(ci_config.clone());
                match generator.generate_workflow(&matrix) {
                    Ok(workflow) => {
                        std::fs::create_dir_all(".github/workflows").ok();
                        match std::fs::write(".github/workflows/forge.yml", workflow) {
                            Ok(_) => println!("✓ Created .github/workflows/forge.yml"),
                            Err(e) => {
                                eprintln!("error: failed to write GitHub Actions workflow: {}", e);
                                return ExitCode::FAILURE;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("error: failed to generate GitHub Actions workflow: {}", e);
                        return ExitCode::FAILURE;
                    }
                }
            }
            
            if ci_config.platform == CIPlatform::GitLabCI || 
               ci_config.platform == CIPlatform::Both {
                let generator = GitLabCIGenerator::new(ci_config.clone());
                match generator.generate_pipeline(&matrix) {
                    Ok(pipeline) => {
                        match std::fs::write(".gitlab-ci.yml", pipeline) {
                            Ok(_) => println!("✓ Created .gitlab-ci.yml"),
                            Err(e) => {
                                eprintln!("error: failed to write GitLab CI pipeline: {}", e);
                                return ExitCode::FAILURE;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("error: failed to generate GitLab CI pipeline: {}", e);
                        return ExitCode::FAILURE;
                    }
                }
            }
            
            println!("✓ CI configuration initialized successfully");
            println!("  Platform: {}", platform);
            println!("  Cache: {}", if cache { "enabled" } else { "disabled" });
            if let Some(url) = &remote_cache {
                println!("  Remote cache: {}", url);
            }
            
            ExitCode::SUCCESS
        }
        CiCommand::Export { output, platform } => {
            let ci_config = CIConfig {
                platform: match platform.as_str() {
                    "github" => CIPlatform::GitHubActions,
                    "gitlab" => CIPlatform::GitLabCI,
                    _ => {
                        eprintln!("error: invalid platform '{}', expected 'github' or 'gitlab'", platform);
                        return ExitCode::FAILURE;
                    }
                },
                cache_enabled: true,
                remote_cache_url: None,
                jobs_per_run: 4,
                timeout_minutes: 30,
            };
            
            // Create a sample matrix for export
            let mut matrix = CIMatrix::new();
            matrix.add_job(CIJob {
                id: "build".to_string(),
                name: "Build".to_string(),
                backend: "rust".to_string(),
                commands: vec!["cargo build".to_string()],
                artifacts: vec![],
                dependencies: vec![],
                cache_key: "cache-key".to_string(),
            });
            
            let result = match platform.as_str() {
                "github" => {
                    let generator = GitHubActionsGenerator::new(ci_config);
                    generator.generate_workflow(&matrix)
                }
                "gitlab" => {
                    let generator = GitLabCIGenerator::new(ci_config);
                    generator.generate_pipeline(&matrix)
                }
                _ => unreachable!(),
            };
            
            match result {
                Ok(content) => {
                    match std::fs::write(&output, content) {
                        Ok(_) => {
                            println!("✓ Exported CI configuration to {}", output.display());
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("error: failed to write to {}: {}", output.display(), e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: failed to generate CI configuration: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
    }
}
