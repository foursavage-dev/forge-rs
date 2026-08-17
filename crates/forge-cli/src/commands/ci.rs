use std::process::ExitCode;

use crate::args::{CiArgs, CiCommand};
use forge_ci_generator::{
    BitbucketPipelineGenerator, CIConfig, CIJob, CIMatrix, CIPlatform, CircleCIGenerator,
    GitHubActionsGenerator, GitLabCIGenerator,
};

pub fn run_ci(args: CiArgs) -> ExitCode {
    match args.command {
        CiCommand::Init {
            platform,
            cache,
            remote_cache,
        } => {
            let ci_config = CIConfig {
                platform: match platform.as_str() {
                    "github" => CIPlatform::GitHubActions,
                    "gitlab" => CIPlatform::GitLabCI,
                    "circleci" => CIPlatform::CircleCI,
                    "bitbucket" => CIPlatform::BitbucketPipelines,
                    "all" => CIPlatform::All,
                    _ => {
                        eprintln!(
                            "error: invalid platform '{}', expected 'github', 'gitlab', 'circleci', 'bitbucket', or 'all'",
                            platform
                        );
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
            match ci_config.platform {
                CIPlatform::GitHubActions => {
                    let generator = GitHubActionsGenerator::new(ci_config.clone());
                    match generator.generate_workflow(&matrix) {
                        Ok(workflow) => {
                            std::fs::create_dir_all(".github/workflows").ok();
                            match std::fs::write(".github/workflows/forge.yml", workflow) {
                                Ok(_) => println!("✓ Created .github/workflows/forge.yml"),
                                Err(e) => {
                                    eprintln!(
                                        "error: failed to write GitHub Actions workflow: {}",
                                        e
                                    );
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
                CIPlatform::GitLabCI => {
                    let generator = GitLabCIGenerator::new(ci_config.clone());
                    match generator.generate_pipeline(&matrix) {
                        Ok(pipeline) => match std::fs::write(".gitlab-ci.yml", pipeline) {
                            Ok(_) => println!("✓ Created .gitlab-ci.yml"),
                            Err(e) => {
                                eprintln!("error: failed to write GitLab CI pipeline: {}", e);
                                return ExitCode::FAILURE;
                            }
                        },
                        Err(e) => {
                            eprintln!("error: failed to generate GitLab CI pipeline: {}", e);
                            return ExitCode::FAILURE;
                        }
                    }
                }
                CIPlatform::CircleCI => {
                    let generator = CircleCIGenerator::new(ci_config.clone());
                    match generator.generate_config(&matrix) {
                        Ok(config) => match std::fs::write(".circleci/config.yml", config) {
                            Ok(_) => println!("✓ Created .circleci/config.yml"),
                            Err(e) => {
                                eprintln!("error: failed to write CircleCI config: {}", e);
                                return ExitCode::FAILURE;
                            }
                        },
                        Err(e) => {
                            eprintln!("error: failed to generate CircleCI config: {}", e);
                            return ExitCode::FAILURE;
                        }
                    }
                }
                CIPlatform::BitbucketPipelines => {
                    let generator = BitbucketPipelineGenerator::new(ci_config.clone());
                    match generator.generate_config(&matrix) {
                        Ok(config) => match std::fs::write("bitbucket-pipelines.yml", config) {
                            Ok(_) => println!("✓ Created bitbucket-pipelines.yml"),
                            Err(e) => {
                                eprintln!(
                                    "error: failed to write Bitbucket Pipelines config: {}",
                                    e
                                );
                                return ExitCode::FAILURE;
                            }
                        },
                        Err(e) => {
                            eprintln!(
                                "error: failed to generate Bitbucket Pipelines config: {}",
                                e
                            );
                            return ExitCode::FAILURE;
                        }
                    }
                }
                CIPlatform::All => {
                    // Generate all configurations
                    let platforms = vec![
                        (CIPlatform::GitHubActions, ".github/workflows/forge.yml"),
                        (CIPlatform::GitLabCI, ".gitlab-ci.yml"),
                        (CIPlatform::CircleCI, ".circleci/config.yml"),
                        (CIPlatform::BitbucketPipelines, "bitbucket-pipelines.yml"),
                    ];

                    for (platform, file_path) in platforms {
                        let config = CIConfig {
                            platform,
                            cache_enabled: cache,
                            remote_cache_url: remote_cache.clone(),
                            jobs_per_run: 4,
                            timeout_minutes: 30,
                        };

                        let result = config.generate_ci(&matrix);
                        match result {
                            Ok(content) => {
                                if let Some(parent_dir) = std::path::Path::new(file_path).parent() {
                                    std::fs::create_dir_all(parent_dir).ok();
                                }
                                match std::fs::write(file_path, content) {
                                    Ok(_) => println!("✓ Created {}", file_path),
                                    Err(e) => {
                                        eprintln!("error: failed to write {}: {}", file_path, e);
                                        return ExitCode::FAILURE;
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("error: failed to generate CI configuration: {}", e);
                                return ExitCode::FAILURE;
                            }
                        }
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
                    "circleci" => CIPlatform::CircleCI,
                    "bitbucket" => CIPlatform::BitbucketPipelines,
                    _ => {
                        eprintln!(
                            "error: invalid platform '{}', expected 'github', 'gitlab', 'circleci', or 'bitbucket'",
                            platform
                        );
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
                "circleci" => {
                    let generator = CircleCIGenerator::new(ci_config);
                    generator.generate_config(&matrix)
                }
                "bitbucket" => {
                    let generator = BitbucketPipelineGenerator::new(ci_config);
                    generator.generate_config(&matrix)
                }
                _ => unreachable!(),
            };

            match result {
                Ok(content) => match std::fs::write(&output, content) {
                    Ok(_) => {
                        println!("✓ Exported CI configuration to {}", output.display());
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("error: failed to write to {}: {}", output.display(), e);
                        ExitCode::FAILURE
                    }
                },
                Err(e) => {
                    eprintln!("error: failed to generate CI configuration: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
    }
}
