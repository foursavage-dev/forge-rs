use std::process::ExitCode;
use forge_cache::LocalCache;
use crate::utils::human_bytes;

pub fn run_doctor() -> ExitCode {
    println!("🦀 Forge Doctor");
    let mut all_ok = true;

    for (tool, version_args) in [
        ("cargo", &["--version"][..]),
        ("git", &["--version"][..]),
    ] {
        match std::process::Command::new(tool).args(version_args).output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                println!("  [ok] {tool} found{version}", version = if version.is_empty() { String::new() } else { format!(": {version}") });
            }
            _ => {
                println!("  [fail] {tool} is not available on PATH");
                all_ok = false;
            }
        }
    }

    match LocalCache::default_location() {
        Ok(cache) => {
            let probe = cache.root().join(".doctor-probe");
            match std::fs::write(&probe, b"ok") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&probe);
                    let stats = cache.disk_stats();
                    println!(
                        "  [ok] cache dir {} is writable ({} records, {} objects, {})",
                        cache.root().display(),
                        stats.record_count,
                        stats.object_count,
                        human_bytes(stats.total_bytes)
                    );
                }
                Err(error) => {
                    println!(
                        "  [fail] cache dir {} is not writable: {error}",
                        cache.root().display()
                    );
                    all_ok = false;
                }
            }
        }
        Err(error) => {
            println!("  [fail] cannot open the local cache: {error}");
            all_ok = false;
        }
    }

    if all_ok {
        println!("All checks passed.");
        ExitCode::SUCCESS
    } else {
        println!("Some checks failed.");
        ExitCode::FAILURE
    }
}
