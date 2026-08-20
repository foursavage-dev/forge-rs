#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkerKind {
    Mold,
    Lld,
    Msvc,
    SystemDefault,
}

pub struct LinkerDispatcher;

impl LinkerDispatcher {
    pub fn detect_best_linker() -> LinkerKind {
        if cfg!(target_os = "linux") {
            if Self::is_executable_in_path("mold") {
                return LinkerKind::Mold;
            }
            if Self::is_executable_in_path("ld.lld") || Self::is_executable_in_path("lld") {
                return LinkerKind::Lld;
            }
        } else if cfg!(target_os = "windows") {
            if Self::is_executable_in_path("lld-link.exe")
                || Self::is_executable_in_path("lld-link")
            {
                return LinkerKind::Lld;
            }
            return LinkerKind::Msvc;
        } else if cfg!(target_os = "macos")
            && (Self::is_executable_in_path("ld64.lld") || Self::is_executable_in_path("lld"))
        {
            return LinkerKind::Lld;
        }

        LinkerKind::SystemDefault
    }

    pub fn rustc_linker_flags(kind: LinkerKind) -> Vec<String> {
        match kind {
            LinkerKind::Mold => vec!["-C".to_string(), "link-arg=-fuse-ld=mold".to_string()],
            LinkerKind::Lld => {
                if cfg!(target_os = "windows") {
                    vec!["-C".to_string(), "link-arg=-fuse-ld=lld-link".to_string()]
                } else {
                    vec!["-C".to_string(), "link-arg=-fuse-ld=lld".to_string()]
                }
            }
            LinkerKind::Msvc => vec![],
            LinkerKind::SystemDefault => vec![],
        }
    }

    pub fn cc_linker_flags(kind: LinkerKind) -> Vec<String> {
        match kind {
            LinkerKind::Mold => vec!["-fuse-ld=mold".to_string()],
            LinkerKind::Lld => vec!["-fuse-ld=lld".to_string()],
            LinkerKind::Msvc => vec![],
            LinkerKind::SystemDefault => vec![],
        }
    }

    fn is_executable_in_path(name: &str) -> bool {
        if let Some(paths) = std::env::var_os("PATH") {
            for path in std::env::split_paths(&paths) {
                let candidate = path.join(name);
                if candidate.is_file() {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linker_dispatcher_flags() {
        let mold_flags = LinkerDispatcher::rustc_linker_flags(LinkerKind::Mold);
        assert_eq!(mold_flags, vec!["-C", "link-arg=-fuse-ld=mold"]);

        let cc_flags = LinkerDispatcher::cc_linker_flags(LinkerKind::Mold);
        assert_eq!(cc_flags, vec!["-fuse-ld=mold"]);

        let def_flags = LinkerDispatcher::rustc_linker_flags(LinkerKind::SystemDefault);
        assert!(def_flags.is_empty());
    }

    #[test]
    fn test_detect_best_linker_runs() {
        let detected = LinkerDispatcher::detect_best_linker();
        assert!(matches!(
            detected,
            LinkerKind::Mold | LinkerKind::Lld | LinkerKind::Msvc | LinkerKind::SystemDefault
        ));
    }
}
