use std::path::PathBuf;
use std::process::ExitCode;

use forge_cache::LocalCache;
use forge_cas::{Artifact, ArtifactHash, CasStorage, CasStorageConfig, CleanupPolicy};

use crate::args::{CacheArgs, CacheCommand, CasArgs, CasCommand};
use crate::utils;

pub fn run_cache(args: CacheArgs) -> ExitCode {
    let cache = match &args.dir {
        Some(dir) => match LocalCache::new(dir) {
            Ok(cache) => cache,
            Err(error) => {
                eprintln!("error: cannot open cache at `{}`: {error}", dir.display());
                return ExitCode::FAILURE;
            }
        },
        None => match LocalCache::default_location() {
            Ok(cache) => cache,
            Err(error) => {
                eprintln!("error: cannot open the default cache: {error}");
                return ExitCode::FAILURE;
            }
        },
    };

    match args.command {
        CacheCommand::Stats => {
            let stats = cache.disk_stats();
            println!("Cache dir:          {}", cache.root().display());
            println!(
                "Fingerprint records: {} ({})",
                stats.record_count,
                utils::human_bytes(stats.fingerprints_bytes)
            );
            println!(
                "Artifact objects:    {} ({})",
                stats.object_count,
                utils::human_bytes(stats.objects_bytes)
            );
            println!(
                "Total:               {}",
                utils::human_bytes(stats.total_bytes)
            );
            ExitCode::SUCCESS
        }
        CacheCommand::Prune(prune) => {
            let older_than = match prune
                .older_than
                .as_deref()
                .map(utils::parse_duration)
                .transpose()
            {
                Ok(value) => value,
                Err(message) => {
                    eprintln!("error: --older-than: {message}");
                    return ExitCode::FAILURE;
                }
            };
            let max_size = match prune.max_size.as_deref().map(utils::parse_size).transpose() {
                Ok(value) => value,
                Err(message) => {
                    eprintln!("error: --max-size: {message}");
                    return ExitCode::FAILURE;
                }
            };
            match cache.prune(older_than, max_size) {
                Ok(report) => {
                    println!(
                        "Removed {} fingerprint records and {} objects (freed {}).",
                        report.removed_records,
                        report.removed_objects,
                        utils::human_bytes(report.freed_bytes)
                    );
                    let stats = cache.disk_stats();
                    println!(
                        "Cache now: {} records, {} objects, {} total.",
                        stats.record_count,
                        stats.object_count,
                        utils::human_bytes(stats.total_bytes)
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("error: prune failed: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        CacheCommand::Cas(cas_args) => run_cas(&cache, cas_args),
    }
}

pub fn run_cas(cache: &LocalCache, args: CasArgs) -> ExitCode {
    let rt = tokio::runtime::Runtime::new().unwrap();

    match args.command {
        CasCommand::Stats => {
            let cas_path = cache.cas_path();
            let config = CasStorageConfig::local(&cas_path);
            match rt.block_on(CasStorage::new(config)) {
                Ok(storage) => match rt.block_on(storage.stats()) {
                    Ok(stats) => {
                        println!("CAS Storage:         {}", cas_path.display());
                        println!("Backend type:        {}", stats.backend_type);
                        println!("Artifacts:          {}", stats.artifact_count);
                        println!(
                            "Total size:          {}",
                            utils::human_bytes(stats.total_bytes)
                        );
                        println!(
                            "Compressed size:    {}",
                            utils::human_bytes(stats.compressed_bytes)
                        );
                        if stats.total_bytes > 0 {
                            let ratio = stats.compressed_bytes as f64 / stats.total_bytes as f64;
                            println!("Compression ratio:   {:.2}%", (1.0 - ratio) * 100.0);
                        }
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("error: failed to get CAS stats: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Err(e) => {
                    eprintln!("error: failed to initialize CAS storage: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        CasCommand::Upload {
            file,
            artifact_type,
            source,
        } => {
            let artifact_type = artifact_type.unwrap_or_else(|| {
                file.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("binary")
                    .to_string()
            });
            let source = source.unwrap_or_else(|| {
                file.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });

            match rt.block_on(Artifact::from_file(&file)) {
                Ok(mut artifact) => {
                    artifact.metadata.artifact_type = artifact_type;
                    artifact.metadata.source = source;

                    let cas_path = cache.cas_path();
                    let config = CasStorageConfig::local(&cas_path);
                    match rt.block_on(CasStorage::new(config)) {
                        Ok(storage) => match rt.block_on(storage.store(&artifact)) {
                            Ok(_) => {
                                println!("Artifact uploaded successfully");
                                println!("Hash: {}", artifact.hash());
                                println!("Size: {}", utils::human_bytes(artifact.size()));
                                if let Some(ratio) = artifact.compression_ratio() {
                                    println!("Compression: {:.2}%", (1.0 - ratio) * 100.0);
                                }
                                ExitCode::SUCCESS
                            }
                            Err(e) => {
                                eprintln!("error: failed to store artifact: {}", e);
                                ExitCode::FAILURE
                            }
                        },
                        Err(e) => {
                            eprintln!("error: failed to initialize CAS storage: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: failed to read artifact file: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        CasCommand::Download { hash, output } => {
            let artifact_hash = ArtifactHash::new(hash.clone());
            let output_path = output.unwrap_or_else(|| {
                // Use hash as filename if no output specified
                PathBuf::from(hash)
            });

            let cas_path = cache.cas_path();
            let config = CasStorageConfig::local(&cas_path);
            match rt.block_on(CasStorage::new(config)) {
                Ok(storage) => match rt.block_on(storage.retrieve(&artifact_hash)) {
                    Ok(artifact) => match std::fs::write(&output_path, artifact.data()) {
                        Ok(_) => {
                            println!("Artifact downloaded successfully");
                            println!("Output: {}", output_path.display());
                            println!("Size: {}", utils::human_bytes(artifact.size()));
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("error: failed to write artifact: {}", e);
                            ExitCode::FAILURE
                        }
                    },
                    Err(e) => {
                        eprintln!("error: failed to retrieve artifact: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Err(e) => {
                    eprintln!("error: failed to initialize CAS storage: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        CasCommand::List => {
            let cas_path = cache.cas_path();
            let config = CasStorageConfig::local(&cas_path);
            match rt.block_on(CasStorage::new(config)) {
                Ok(storage) => match rt.block_on(storage.list()) {
                    Ok(hashes) => {
                        println!("CAS Artifacts ({} total):", hashes.len());
                        for hash in hashes {
                            println!("  {}", hash);
                        }
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("error: failed to list artifacts: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Err(e) => {
                    eprintln!("error: failed to initialize CAS storage: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        CasCommand::Delete { hash } => {
            let artifact_hash = ArtifactHash::new(hash);
            let cas_path = cache.cas_path();
            let config = CasStorageConfig::local(&cas_path);
            match rt.block_on(CasStorage::new(config)) {
                Ok(storage) => match rt.block_on(storage.delete(&artifact_hash)) {
                    Ok(_) => {
                        println!("Artifact deleted successfully");
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("error: failed to delete artifact: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Err(e) => {
                    eprintln!("error: failed to initialize CAS storage: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        CasCommand::Cleanup {
            older_than,
            max_size,
        } => {
            let older_than_duration =
                match older_than.as_deref().map(utils::parse_duration).transpose() {
                    Ok(value) => value,
                    Err(message) => {
                        eprintln!("error: --older-than: {message}");
                        return ExitCode::FAILURE;
                    }
                };
            let max_size_bytes = match max_size.as_deref().map(utils::parse_size).transpose() {
                Ok(value) => value,
                Err(message) => {
                    eprintln!("error: --max-size: {message}");
                    return ExitCode::FAILURE;
                }
            };

            let cas_path = cache.cas_path();
            let config =
                CasStorageConfig::local(&cas_path).with_max_size(max_size_bytes.unwrap_or(0));
            match rt.block_on(CasStorage::new(config)) {
                Ok(storage) => {
                    let policy = if let Some(duration) = older_than_duration {
                        CleanupPolicy::OlderThan(duration)
                    } else {
                        CleanupPolicy::OlderThan(std::time::Duration::from_secs(7 * 24 * 60 * 60)) // 7 days default
                    };

                    match rt.block_on(storage.cleanup(policy)) {
                        Ok(result) => {
                            println!("Removed {} artifacts", result.removed_count);
                            println!("Freed {}", utils::human_bytes(result.freed_bytes));
                            if let Some(max_bytes) = max_size_bytes {
                                println!("Max size limit: {}", utils::human_bytes(max_bytes));
                            }
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("error: failed to cleanup CAS: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: failed to initialize CAS storage: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
    }
}
