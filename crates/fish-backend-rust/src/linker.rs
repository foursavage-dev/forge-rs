use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastLinker {
    Mold,
    Lld,
    LldLink,
    Sold,
    Msvc,
    GnuLd,
}

impl FastLinker {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mold => "mold",
            Self::Lld => "lld",
            Self::LldLink => "lld-link",
            Self::Sold => "sold",
            Self::Msvc => "link.exe",
            Self::GnuLd => "ld",
        }
    }

    pub fn to_rustflags(&self) -> Vec<String> {
        match self {
            Self::Mold => vec!["-C".to_string(), "link-arg=-fuse-ld=mold".to_string()],
            Self::Sold => vec!["-C".to_string(), "link-arg=-fuse-ld=sold".to_string()],
            Self::Lld => vec!["-C".to_string(), "link-arg=-fuse-ld=lld".to_string()],
            Self::LldLink => vec!["-C".to_string(), "linker=lld-link".to_string()],
            Self::Msvc => vec![],
            Self::GnuLd => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct RustLinkerOptimizer {
    pub preferred_linker: FastLinker,
    pub enable_split_debuginfo: bool,
    pub parallel_codegen_units: usize,
    pub use_cranelift: bool,
}

impl RustLinkerOptimizer {
    pub fn auto_detect() -> Self {
        let preferred_linker = Self::detect_best_linker();
        let parallel_codegen_units = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(8);

        Self {
            preferred_linker,
            enable_split_debuginfo: cfg!(target_os = "macos") || cfg!(target_os = "windows"),
            parallel_codegen_units,
            use_cranelift: false,
        }
    }

    pub fn detect_best_linker() -> FastLinker {
        if cfg!(target_os = "linux") {
            if Self::is_command_available("mold") {
                return FastLinker::Mold;
            }
            if Self::is_command_available("lld") {
                return FastLinker::Lld;
            }
        } else if cfg!(target_os = "macos") {
            if Self::is_command_available("sold") {
                return FastLinker::Sold;
            }
            if Self::is_command_available("lld") {
                return FastLinker::Lld;
            }
        } else if cfg!(target_os = "windows") && Self::is_command_available("lld-link") {
            return FastLinker::LldLink;
        }

        if cfg!(target_os = "windows") {
            FastLinker::Msvc
        } else {
            FastLinker::GnuLd
        }
    }

    fn is_command_available(cmd: &str) -> bool {
        if let Ok(path_var) = std::env::var("PATH") {
            let separator = if cfg!(target_os = "windows") {
                ';'
            } else {
                ':'
            };
            for dir in path_var.split(separator) {
                let candidate = PathBuf::from(dir).join(cmd);
                if candidate.is_file() {
                    return true;
                }
                if cfg!(target_os = "windows") {
                    let exe_candidate = PathBuf::from(dir).join(format!("{cmd}.exe"));
                    if exe_candidate.is_file() {
                        return true;
                    }
                }
            }
        }

        Command::new(cmd)
            .arg("--version")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    pub fn generate_rustflags(&self, is_release: bool) -> Vec<String> {
        let mut flags = self.preferred_linker.to_rustflags();

        if !is_release {
            flags.push("-C".to_string());
            flags.push(format!("codegen-units={}", self.parallel_codegen_units));

            if self.enable_split_debuginfo {
                flags.push("-C".to_string());
                flags.push("split-debuginfo=unpacked".to_string());
            }

            if self.use_cranelift {
                flags.push("-Zcodegen-backend=cranelift".to_string());
            }
        }

        flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linker_flags_generation() {
        let mold_flags = FastLinker::Mold.to_rustflags();
        assert_eq!(mold_flags, vec!["-C", "link-arg=-fuse-ld=mold"]);

        let lld_link_flags = FastLinker::LldLink.to_rustflags();
        assert_eq!(lld_link_flags, vec!["-C", "linker=lld-link"]);
    }

    #[test]
    fn test_optimizer_rustflags() {
        let optimizer = RustLinkerOptimizer {
            preferred_linker: FastLinker::Mold,
            enable_split_debuginfo: true,
            parallel_codegen_units: 4,
            use_cranelift: false,
        };

        let debug_flags = optimizer.generate_rustflags(false);
        assert!(debug_flags.contains(&"codegen-units=4".to_string()));
        assert!(debug_flags.contains(&"link-arg=-fuse-ld=mold".to_string()));
        assert!(debug_flags.contains(&"split-debuginfo=unpacked".to_string()));

        let release_flags = optimizer.generate_rustflags(true);
        assert_eq!(release_flags, vec!["-C", "link-arg=-fuse-ld=mold"]);
    }
}
