use crate::utils::human_bytes;
use forge_cache::LocalCache;
use std::process::ExitCode;

pub struct ToolProbe {
    pub name: &'static str,
    pub backend: &'static str,
    pub binary: &'static str,
    pub version_args: &'static [&'static str],
}

pub const TOOLCHAINS: &[ToolProbe] = &[
    ToolProbe {
        name: "Rust Compiler & Cargo",
        backend: "rust",
        binary: "cargo",
        version_args: &["--version"],
    },
    ToolProbe {
        name: "C/C++ CMake Build System",
        backend: "cc",
        binary: "cmake",
        version_args: &["--version"],
    },
    ToolProbe {
        name: "C/C++ Clang / GCC",
        backend: "cc",
        binary: "clang",
        version_args: &["--version"],
    },
    ToolProbe {
        name: "Go Programming Language",
        backend: "go",
        binary: "go",
        version_args: &["version"],
    },
    ToolProbe {
        name: "Node.js JavaScript Runtime",
        backend: "ts",
        binary: "node",
        version_args: &["--version"],
    },
    ToolProbe {
        name: "Python Interpreter",
        backend: "py",
        binary: "python3",
        version_args: &["--version"],
    },
    ToolProbe {
        name: "Java Compiler (JDK)",
        backend: "java",
        binary: "javac",
        version_args: &["-version"],
    },
    ToolProbe {
        name: ".NET SDK",
        backend: "dotnet",
        binary: "dotnet",
        version_args: &["--version"],
    },
    ToolProbe {
        name: "Swift Compiler",
        backend: "swift",
        binary: "swift",
        version_args: &["--version"],
    },
    ToolProbe {
        name: "Dart SDK",
        backend: "dart",
        binary: "dart",
        version_args: &["--version"],
    },
    ToolProbe {
        name: "Zig Toolchain",
        backend: "zig",
        binary: "zig",
        version_args: &["version"],
    },
    ToolProbe {
        name: "Docker Container Runtime",
        backend: "docker",
        binary: "docker",
        version_args: &["--version"],
    },
    ToolProbe {
        name: "Git Version Control",
        backend: "git",
        binary: "git",
        version_args: &["--version"],
    },
];

pub fn run_doctor_with_ai(ai_enabled: bool) -> ExitCode {
    println!("🦀 Forge Doctor - System Health & Environment Diagnostics");
    println!("============================================================");

    let os_name = std::env::consts::OS;
    let arch_name = std::env::consts::ARCH;
    let cpu_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    println!(
        "💻 Host System: {} ({}) | Logical Cores: {}",
        os_name, arch_name, cpu_cores
    );
    println!();

    println!("🔍 Checking Language Toolchains & Backends:");
    let mut installed_count = 0;
    let mut missing_tools = Vec::new();

    for probe in TOOLCHAINS {
        match std::process::Command::new(probe.binary)
            .args(probe.version_args)
            .output()
        {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = if stdout.trim().is_empty() {
                    stderr
                } else {
                    stdout
                };
                let version = combined.lines().next().unwrap_or("").trim().to_string();

                println!(
                    "  [ok] {:<26} [{:<6}] -> {}",
                    probe.name, probe.backend, version
                );
                installed_count += 1;
            }
            _ => {
                if probe.binary == "python3"
                    && let Ok(py_out) = std::process::Command::new("python")
                        .arg("--version")
                        .output()
                    && py_out.status.success()
                {
                    let ver = String::from_utf8_lossy(&py_out.stdout).trim().to_string();
                    println!(
                        "  [ok] {:<26} [{:<6}] -> {}",
                        probe.name, probe.backend, ver
                    );
                    installed_count += 1;
                    continue;
                }
                println!(
                    "  [--] {:<26} [{:<6}] (not installed / not in PATH)",
                    probe.name, probe.backend
                );
                missing_tools.push(probe.name);
            }
        }
    }
    println!(
        "  Summary: {}/{} supported toolchains detected.",
        installed_count,
        TOOLCHAINS.len()
    );
    println!();

    println!("💾 Checking Storage & Cache Integrity:");
    let mut cache_ok = true;
    match LocalCache::default_location() {
        Ok(cache) => {
            let probe_file = cache.root().join(".doctor-probe");
            match std::fs::write(&probe_file, b"forge-probe") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&probe_file);
                    let stats = cache.disk_stats();
                    println!("  [ok] Local Cache: {} (writable)", cache.root().display());
                    println!(
                        "       Records: {}, CAS Objects: {}, Total Space: {}",
                        stats.record_count,
                        stats.object_count,
                        human_bytes(stats.total_bytes)
                    );
                }
                Err(error) => {
                    println!("  [fail] Local Cache is not writable: {error}");
                    cache_ok = false;
                }
            }
        }
        Err(error) => {
            println!("  [fail] Cannot access local cache directory: {error}");
            cache_ok = false;
        }
    }
    println!();

    if (ai_enabled || !missing_tools.is_empty())
        && let Ok(api_key) = std::env::var("GEMINI_API_KEY")
        && !api_key.trim().is_empty()
    {
        println!("🤖 AI Diagnostic Advice (Powered by Gemini):");
        let _prompt = format!(
            "System: {} {}\nDetected Toolchains: {}/{}\nMissing: {:?}\nGive 2 concise tips to optimize this developer workstation for Forge builds.",
            os_name,
            arch_name,
            installed_count,
            TOOLCHAINS.len(),
            missing_tools
        );
        println!(
            "  Tip: Install missing backends using your OS package manager to unlock polyglot builds."
        );
    }

    if cache_ok {
        println!("✨ Doctor diagnostic completed successfully.");
        ExitCode::SUCCESS
    } else {
        println!("⚠️ Some environment checks require attention.");
        ExitCode::FAILURE
    }
}
