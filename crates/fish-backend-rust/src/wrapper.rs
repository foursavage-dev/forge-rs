use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustcInvocation {
    pub crate_name: String,
    pub crate_type: String,
    pub edition: String,
    pub out_dir: Option<PathBuf>,
    pub output_file: Option<PathBuf>,
    pub source_files: Vec<PathBuf>,
    pub externs: BTreeMap<String, PathBuf>,
    pub compiler_flags: Vec<String>,
}

impl RustcInvocation {
    pub fn parse_from_args(args: &[String]) -> Self {
        let mut crate_name = String::new();
        let mut crate_type = "lib".to_string();
        let mut edition = "2021".to_string();
        let mut out_dir = None;
        let mut output_file = None;
        let mut source_files = Vec::new();
        let mut externs = BTreeMap::new();
        let mut compiler_flags = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            if arg == "--crate-name" && i + 1 < args.len() {
                crate_name = args[i + 1].clone();
                i += 2;
            } else if arg == "--crate-type" && i + 1 < args.len() {
                crate_type = args[i + 1].clone();
                i += 2;
            } else if arg == "--edition" && i + 1 < args.len() {
                edition = args[i + 1].clone();
                i += 2;
            } else if arg == "--out-dir" && i + 1 < args.len() {
                out_dir = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            } else if arg == "-o" && i + 1 < args.len() {
                output_file = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            } else if arg == "--extern" && i + 1 < args.len() {
                let spec = &args[i + 1];
                if let Some((name, path)) = spec.split_once('=') {
                    externs.insert(name.to_string(), PathBuf::from(path));
                }
                i += 2;
            } else if arg.ends_with(".rs") {
                source_files.push(PathBuf::from(arg));
                i += 1;
            } else {
                compiler_flags.push(arg.clone());
                i += 1;
            }
        }

        Self {
            crate_name,
            crate_type,
            edition,
            out_dir,
            output_file,
            source_files,
            externs,
            compiler_flags,
        }
    }

    pub fn compute_invocation_hash(&self, compiler_version: &str) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(compiler_version.as_bytes());
        hasher.update(self.crate_name.as_bytes());
        hasher.update(self.crate_type.as_bytes());
        hasher.update(self.edition.as_bytes());

        for flag in &self.compiler_flags {
            hasher.update(flag.as_bytes());
        }

        for (name, path) in &self.externs {
            hasher.update(name.as_bytes());
            if let Ok(content) = std::fs::read(path) {
                hasher.update(&content);
            } else {
                hasher.update(path.to_string_lossy().as_bytes());
            }
        }

        for src in &self.source_files {
            if let Ok(content) = std::fs::read(src) {
                hasher.update(&content);
            } else {
                hasher.update(src.to_string_lossy().as_bytes());
            }
        }

        hasher.finalize().to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rustc_args() {
        let args = vec![
            "src/lib.rs".to_string(),
            "--crate-name".to_string(),
            "foo".to_string(),
            "--crate-type".to_string(),
            "rlib".to_string(),
            "--edition".to_string(),
            "2024".to_string(),
            "--extern".to_string(),
            "bar=/path/to/libbar.rlib".to_string(),
            "-C".to_string(),
            "opt-level=3".to_string(),
        ];

        let inv = RustcInvocation::parse_from_args(&args);
        assert_eq!(inv.crate_name, "foo");
        assert_eq!(inv.crate_type, "rlib");
        assert_eq!(inv.edition, "2024");
        assert_eq!(inv.source_files, vec![PathBuf::from("src/lib.rs")]);
        assert_eq!(
            inv.externs.get("bar"),
            Some(&PathBuf::from("/path/to/libbar.rlib"))
        );
        assert!(inv.compiler_flags.contains(&"opt-level=3".to_string()));
    }

    #[test]
    fn test_deterministic_invocation_hash() {
        let args = vec![
            "src/lib.rs".to_string(),
            "--crate-name".to_string(),
            "foo".to_string(),
        ];
        let inv = RustcInvocation::parse_from_args(&args);
        let h1 = inv.compute_invocation_hash("rustc 1.88.0");
        let h2 = inv.compute_invocation_hash("rustc 1.88.0");
        assert_eq!(h1, h2);

        let h3 = inv.compute_invocation_hash("rustc 1.89.0");
        assert_ne!(h1, h3);
    }
}
