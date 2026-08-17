use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

pub const DEFAULT_EXCLUDED_DIRS: &[&str] = &[
    ".git",
    ".forge",
    "target",
    "build",
    "dist",
    "node_modules",
    "vendor",
    ".dart_tool",
    "__pycache__",
    ".venv",
    "venv",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    "bin",
    "obj",
    ".gradle",
    ".m2",
    ".idea",
    ".vscode",
    ".next",
    ".turbo",
    "zig-cache",
    "zig-out",
];

pub struct FingerprintUtils;

impl FingerprintUtils {
    pub fn compute_namespace(path: &Path) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(path.to_string_lossy().as_bytes());
        hasher.finalize().to_hex().to_string()[..12].to_string()
    }

    pub fn combine_fingerprints(prefix: &str, fingerprints: &[String]) -> String {
        let mut hasher = blake3::Hasher::new();
        if !prefix.is_empty() {
            hasher.update(format!("prefix[{}]:", prefix.len()).as_bytes());
            hasher.update(prefix.as_bytes());
        }
        let mut sorted = fingerprints.to_vec();
        sorted.sort();
        for fp in &sorted {
            hasher.update(format!("fp[{}]:", fp.len()).as_bytes());
            hasher.update(fp.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }

    pub fn format_cache_key(
        backend: &str,
        namespace: &str,
        mode: &str,
        identifier: &str,
    ) -> String {
        format!("{}/{}/{}/{}", backend, namespace, mode, identifier)
    }

    pub fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        hasher.finalize().to_hex().to_string()
    }

    pub fn hash_file_into(path: &Path, hasher: &mut blake3::Hasher) -> Result<(), io::Error> {
        let mut file = File::open(path)?;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(())
    }

    pub fn hash_file(path: &Path) -> Result<String, io::Error> {
        let mut hasher = blake3::Hasher::new();
        Self::hash_file_into(path, &mut hasher)?;
        Ok(hasher.finalize().to_hex().to_string())
    }

    pub fn hash_directory_filtered<P, F1, F2>(
        dir: P,
        is_excluded_dir: F1,
        is_allowed_file: F2,
        hasher: &mut blake3::Hasher,
    ) -> Result<(), io::Error>
    where
        P: AsRef<Path>,
        F1: Fn(&str) -> bool,
        F2: Fn(&Path) -> bool,
    {
        fn walk<F1, F2>(
            current: &Path,
            base: &Path,
            is_excluded_dir: &F1,
            is_allowed_file: &F2,
            hasher: &mut blake3::Hasher,
        ) -> Result<(), io::Error>
        where
            F1: Fn(&str) -> bool,
            F2: Fn(&Path) -> bool,
        {
            if !current.exists() || !current.is_dir() {
                return Ok(());
            }

            let mut entries = fs::read_dir(current)?
                .filter_map(|e| e.ok())
                .collect::<Vec<_>>();
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let path = entry.path();
                let file_name = entry.file_name();
                let name_str = file_name.to_string_lossy();

                if path.is_dir() {
                    if is_excluded_dir(&name_str) {
                        continue;
                    }
                    walk(&path, base, is_excluded_dir, is_allowed_file, hasher)?;
                } else if path.is_file() && is_allowed_file(&path) {
                    let rel = path.strip_prefix(base).unwrap_or(&path);
                    hasher.update(rel.to_string_lossy().as_bytes());
                    hasher.update(b":");
                    FingerprintUtils::hash_file_into(&path, hasher)?;
                }
            }
            Ok(())
        }

        let dir_ref = dir.as_ref();
        walk(dir_ref, dir_ref, &is_excluded_dir, &is_allowed_file, hasher)
    }

    pub fn hash_directory_with_extensions(
        dir: &Path,
        extensions: &[&str],
        excluded_dirs: &[&str],
        hasher: &mut blake3::Hasher,
    ) -> Result<(), io::Error> {
        Self::hash_directory_filtered(
            dir,
            |d| excluded_dirs.contains(&d) || DEFAULT_EXCLUDED_DIRS.contains(&d),
            |p| {
                if extensions.is_empty() {
                    return true;
                }
                p.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| extensions.contains(&ext))
                    .unwrap_or(false)
            },
            hasher,
        )
    }
}

pub struct ToolchainUtils;

impl ToolchainUtils {
    pub fn get_tool_version(tool: &str, args: &[&str]) -> Result<String, String> {
        let output = std::process::Command::new(tool)
            .args(args)
            .output()
            .map_err(|e| format!("failed to run `{tool}`: {e}"))?;

        if !output.status.success() {
            return Err(format!("`{tool}` exited with {}", output.status));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let first_line = stdout.lines().next().unwrap_or_default().trim();
        if first_line.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(stderr.lines().next().unwrap_or_default().trim().to_string())
        } else {
            Ok(first_line.to_string())
        }
    }

    pub fn tool_available(tool: &str) -> bool {
        std::process::Command::new(tool)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn find_first_available(tools: &[&str]) -> Option<String> {
        tools
            .iter()
            .find(|tool| Self::tool_available(tool))
            .map(|s| s.to_string())
    }

    pub fn resolve_executable(tool: &str) -> Option<PathBuf> {
        let path_var = std::env::var_os("PATH")?;
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(tool);
            if candidate.is_file() {
                return Some(candidate);
            }
            if cfg!(windows) {
                let candidate_exe = dir.join(format!("{tool}.exe"));
                if candidate_exe.is_file() {
                    return Some(candidate_exe);
                }
                let candidate_cmd = dir.join(format!("{tool}.cmd"));
                if candidate_cmd.is_file() {
                    return Some(candidate_cmd);
                }
                let candidate_bat = dir.join(format!("{tool}.bat"));
                if candidate_bat.is_file() {
                    return Some(candidate_bat);
                }
            }
        }
        None
    }
}

pub struct BinaryUtils;

impl BinaryUtils {
    pub fn binary_extension() -> &'static str {
        if cfg!(windows) { ".exe" } else { "" }
    }

    pub fn add_binary_extension(name: &str) -> String {
        if cfg!(windows) && !name.ends_with(".exe") {
            format!("{}.exe", name)
        } else {
            name.to_string()
        }
    }

    pub fn object_extension(is_msvc: bool) -> &'static str {
        if is_msvc { "obj" } else { "o" }
    }

    pub fn static_lib_extension(is_msvc: bool) -> &'static str {
        if is_msvc { "lib" } else { "a" }
    }

    pub fn shared_lib_extension() -> &'static str {
        if cfg!(windows) {
            ".dll"
        } else if cfg!(target_os = "macos") {
            ".dylib"
        } else {
            ".so"
        }
    }
}

pub struct TaskDagBuilder;

impl TaskDagBuilder {
    pub fn resolve_dag_order<T, FKey, FDeps>(
        tasks: &[T],
        key_fn: FKey,
        deps_fn: FDeps,
    ) -> Result<Vec<usize>, String>
    where
        FKey: Fn(&T) -> &str,
        FDeps: Fn(&T) -> &[String],
    {
        let mut key_to_index: HashMap<&str, usize> = HashMap::new();
        for (i, task) in tasks.iter().enumerate() {
            let key = key_fn(task);
            key_to_index.insert(key, i);
        }

        for task in tasks {
            let key = key_fn(task);
            for dep in deps_fn(task) {
                if !key_to_index.contains_key(dep.as_str()) {
                    return Err(format!("task `{key}` depends on unknown task `{dep}`"));
                }
            }
        }

        let mut resolved = Vec::new();
        let mut resolved_set = HashSet::new();

        while resolved.len() < tasks.len() {
            let mut progress = false;
            for (idx, task) in tasks.iter().enumerate() {
                if resolved_set.contains(&idx) {
                    continue;
                }
                let deps = deps_fn(task);
                let all_deps_ready = deps.iter().all(|dep| {
                    if let Some(&dep_idx) = key_to_index.get(dep.as_str()) {
                        resolved_set.contains(&dep_idx)
                    } else {
                        false
                    }
                });

                if all_deps_ready {
                    resolved.push(idx);
                    resolved_set.insert(idx);
                    progress = true;
                }
            }

            if !progress {
                let remaining: Vec<&str> = tasks
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| !resolved_set.contains(idx))
                    .map(|(_, t)| key_fn(t))
                    .collect();
                return Err(format!(
                    "dependency cycle detected among tasks: {}",
                    remaining.join(" -> ")
                ));
            }
        }

        Ok(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_generation() {
        let path = Path::new("/some/path");
        let namespace = FingerprintUtils::compute_namespace(path);
        assert_eq!(namespace.len(), 12);
    }

    #[test]
    fn test_fingerprint_combination() {
        let fps = vec!["fp1".to_string(), "fp2".to_string()];
        let combined = FingerprintUtils::combine_fingerprints("prefix", &fps);
        assert!(!combined.is_empty());
        assert!(combined.len() > 12);
    }

    #[test]
    fn test_cache_key_formatting() {
        let key = FingerprintUtils::format_cache_key("rust", "namespace", "build", "package");
        assert_eq!(key, "rust/namespace/build/package");
    }

    #[test]
    fn test_binary_extension() {
        let ext = BinaryUtils::binary_extension();
        #[cfg(windows)]
        assert_eq!(ext, ".exe");
        #[cfg(not(windows))]
        assert_eq!(ext, "");
    }

    #[test]
    fn test_add_binary_extension() {
        let with_ext = BinaryUtils::add_binary_extension("myapp");
        #[cfg(windows)]
        assert_eq!(with_ext, "myapp.exe");
        #[cfg(not(windows))]
        assert_eq!(with_ext, "myapp");
    }

    #[test]
    fn test_hash_directory_filtered() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let file1 = src_dir.join("a.rs");
        let file2 = src_dir.join("b.txt");
        fs::write(&file1, "fn main() {}").unwrap();
        fs::write(&file2, "ignored").unwrap();

        let mut hasher1 = blake3::Hasher::new();
        FingerprintUtils::hash_directory_with_extensions(
            temp.path(),
            &["rs"],
            &["target"],
            &mut hasher1,
        )
        .unwrap();
        let fp1 = hasher1.finalize().to_hex().to_string();

        let mut hasher2 = blake3::Hasher::new();
        FingerprintUtils::hash_directory_with_extensions(
            temp.path(),
            &["rs"],
            &["target"],
            &mut hasher2,
        )
        .unwrap();
        let fp2 = hasher2.finalize().to_hex().to_string();

        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_dag_resolver() {
        struct MockTask {
            name: String,
            deps: Vec<String>,
        }

        let tasks = vec![
            MockTask {
                name: "b".to_string(),
                deps: vec!["a".to_string()],
            },
            MockTask {
                name: "a".to_string(),
                deps: vec![],
            },
        ];

        let order = TaskDagBuilder::resolve_dag_order(&tasks, |t| &t.name, |t| &t.deps).unwrap();
        assert_eq!(order, vec![1, 0]);
    }
}
