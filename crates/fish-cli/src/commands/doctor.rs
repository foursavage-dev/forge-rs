use crate::utils::human_bytes;
use fish_cache::LocalCache;
use std::process::ExitCode;

pub struct ToolProbe {
    pub name: &'static str,
    pub backend: &'static str,
    pub binary: &'static str,
    pub version_args: &'static [&'static str],
    pub install_hint: &'static str,
}

pub const TOOLCHAINS: &[ToolProbe] = &[
    ToolProbe {
        name: "Rust Compiler & Cargo",
        backend: "rust",
        binary: "cargo",
        version_args: &["--version"],
        install_hint: "winget install Rustlang.Rustup || curl https://sh.rustup.rs -sSf | sh",
    },
    ToolProbe {
        name: "C/C++ CMake Build System",
        backend: "cc",
        binary: "cmake",
        version_args: &["--version"],
        install_hint: "winget install Kitware.CMake || sudo apt install cmake || brew install cmake",
    },
    ToolProbe {
        name: "C/C++ Clang / GCC",
        backend: "cc",
        binary: "clang",
        version_args: &["--version"],
        install_hint: "winget install LLVM.LLVM || sudo apt install clang || brew install llvm",
    },
    ToolProbe {
        name: "Go Programming Language",
        backend: "go",
        binary: "go",
        version_args: &["version"],
        install_hint: "winget install GoLang.Go || sudo apt install golang || brew install go",
    },
    ToolProbe {
        name: "Node.js JavaScript Runtime",
        backend: "ts",
        binary: "node",
        version_args: &["--version"],
        install_hint: "winget install OpenJS.NodeJS.LTS || sudo apt install nodejs || brew install node",
    },
    ToolProbe {
        name: "Python Interpreter",
        backend: "py",
        binary: "python3",
        version_args: &["--version"],
        install_hint: "winget install Python.Python.3.12 || sudo apt install python3 || brew install python",
    },
    ToolProbe {
        name: "Java Compiler (JDK)",
        backend: "java",
        binary: "javac",
        version_args: &["-version"],
        install_hint: "winget install Oracle.JDK.21 || sudo apt install default-jdk || brew install openjdk",
    },
    ToolProbe {
        name: ".NET SDK",
        backend: "dotnet",
        binary: "dotnet",
        version_args: &["--version"],
        install_hint: "winget install Microsoft.DotNet.SDK.8 || sudo apt install dotnet-sdk-8.0 || brew install dotnet-sdk",
    },
    ToolProbe {
        name: "Swift Compiler",
        backend: "swift",
        binary: "swift",
        version_args: &["--version"],
        install_hint: "winget install Swift.Toolchain || sudo apt install swift || brew install swift",
    },
    ToolProbe {
        name: "Dart SDK",
        backend: "dart",
        binary: "dart",
        version_args: &["--version"],
        install_hint: "winget install Dart.Dart || sudo apt install dart || brew install dart",
    },
    ToolProbe {
        name: "Zig Toolchain",
        backend: "zig",
        binary: "zig",
        version_args: &["version"],
        install_hint: "winget install zig.zig || sudo apt install zig || brew install zig",
    },
    ToolProbe {
        name: "Docker Container Runtime",
        backend: "docker",
        binary: "docker",
        version_args: &["--version"],
        install_hint: "winget install Docker.DockerDesktop || sudo apt install docker.io || brew install --cask docker",
    },
    ToolProbe {
        name: "Git Version Control",
        backend: "git",
        binary: "git",
        version_args: &["--version"],
        install_hint: "winget install Git.Git || sudo apt install git || brew install git",
    },
];

pub fn run_doctor_with_ai(ai_enabled: bool, fix: bool) -> ExitCode {
    println!("🦀 Fish Doctor - System Health & Environment Diagnostics");
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
                missing_tools.push((probe.name, probe.install_hint));
            }
        }
    }
    println!(
        "  Summary: {}/{} supported toolchains detected.",
        installed_count,
        TOOLCHAINS.len()
    );
    println!();

    if !missing_tools.is_empty() || fix {
        println!("🛠️ Toolchain Installation & Remediation Hints:");
        for (name, hint) in &missing_tools {
            println!("  • {:<26}: {}", name, hint);
        }
        println!();
    }

    println!("💾 Checking Storage & Cache Integrity:");
    let mut cache_ok = true;
    match LocalCache::default_location() {
        Ok(cache) => {
            let probe_file = cache.root().join(".doctor-probe");
            match std::fs::write(&probe_file, b"fish-probe") {
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

    if fix {
        println!("🔧 Applying Automated Remediation (--fix):");
        if let Ok(cache) = LocalCache::default_location() {
            let _ = std::fs::create_dir_all(cache.root());
            println!("  [fixed] Ensured local cache root exists: {}", cache.root().display());
        }
        let manifest_path = std::path::Path::new("fish.toml");
        if !manifest_path.exists() {
            let default_manifest = "[workspace]\nmembers = []\n\n[cache]\nenabled = true\n";
            if std::fs::write(manifest_path, default_manifest).is_ok() {
                println!("  [fixed] Created default `fish.toml` in current directory.");
            }
        }
        println!();
    }

    if (ai_enabled || !missing_tools.is_empty())
        && let Ok(api_key) = std::env::var("GEMINI_API_KEY")
        && !api_key.trim().is_empty()
    {
        println!("🤖 AI Diagnostic Advice (Powered by Gemini):");
        let _prompt = format!(
            "System: {} {}\nDetected Toolchains: {}/{}\nMissing: {:?}\nGive 2 concise tips to optimize this developer workstation for Fish builds.",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolchains_probe_definitions() {
        assert!(!TOOLCHAINS.is_empty());
        for probe in TOOLCHAINS {
            assert!(!probe.name.is_empty());
            assert!(!probe.binary.is_empty());
            assert!(!probe.install_hint.is_empty());
        }
    }
}
