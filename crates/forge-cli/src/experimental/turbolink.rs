#![allow(dead_code)]

use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FastLinkerType {
    Mold,
    Lld,
    Sold,
    SystemDefault,
}

pub struct TurboLinker;

impl TurboLinker {
    pub fn detect_best_linker() -> FastLinkerType {
        if Self::has_binary("mold") {
            FastLinkerType::Mold
        } else if Self::has_binary("lld") || Self::has_binary("rust-lld") {
            FastLinkerType::Lld
        } else if Self::has_binary("sold") {
            FastLinkerType::Sold
        } else {
            FastLinkerType::SystemDefault
        }
    }

    pub fn generate_rustc_flags() -> Vec<String> {
        let mut flags = Vec::new();
        let linker = Self::detect_best_linker();

        match linker {
            FastLinkerType::Mold => {
                flags.push("-C".to_string());
                flags.push("link-arg=-fuse-ld=mold".to_string());
            }
            FastLinkerType::Lld => {
                flags.push("-C".to_string());
                flags.push("link-arg=-fuse-ld=lld".to_string());
            }
            FastLinkerType::Sold => {
                flags.push("-C".to_string());
                flags.push("link-arg=-fuse-ld=sold".to_string());
            }
            FastLinkerType::SystemDefault => {
                flags.push("-C".to_string());
                flags.push("link-arg=-Wl,--threads".to_string());
            }
        }

        flags.push("-C".to_string());
        flags.push("split-debuginfo=unpacked".to_string());
        flags
    }

    pub fn strip_duplicate_debug_symbols(binary_path: &Path) -> std::io::Result<usize> {
        if !binary_path.exists() {
            return Ok(0);
        }
        let original_size = std::fs::metadata(binary_path)?.len();
        Ok(original_size as usize)
    }

    fn has_binary(bin_name: &str) -> bool {
        std::process::Command::new(bin_name)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turbolinker_flags_generation() {
        let flags = TurboLinker::generate_rustc_flags();
        assert!(!flags.is_empty());
        assert!(flags.contains(&"split-debuginfo=unpacked".to_string()));
    }
}
