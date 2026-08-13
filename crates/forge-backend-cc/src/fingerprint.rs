use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn compute_source_fingerprint(
    source_path: &Path,
    includes: &[PathBuf],
    flags: &[String],
    compiler_version: &str,
) -> Result<String, std::io::Error> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(compiler_version.as_bytes());
    hasher.update(b"\nflags:\n");
    for flag in flags {
        hasher.update(flag.as_bytes());
        hasher.update(b"\n");
    }

    if source_path.exists() {
        let content = fs::read(source_path)?;
        hasher.update(&content);

        let parent = source_path.parent();
        let mut visited = HashSet::new();
        scan_and_hash_headers(&content, parent, includes, &mut hasher, &mut visited)?;
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Hash the contents of every `#include`d header, transitively.
///
/// Header includes are followed recursively (with a visited set to stop
/// cycles), so a change in `a.h` that only `b.h` includes still invalidates
/// the task. Headers that cannot be resolved — e.g. system headers like
/// `<stdio.h>` — are skipped, matching the compiler's own search order
/// (source directory first, then the configured include directories).
fn scan_and_hash_headers(
    content: &[u8],
    source_dir: Option<&Path>,
    includes: &[PathBuf],
    hasher: &mut blake3::Hasher,
    visited: &mut HashSet<PathBuf>,
) -> Result<(), std::io::Error> {
    let text = String::from_utf8_lossy(content);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#include") {
            if let Some(header_name) = extract_header_name(trimmed) {
                if let Some(path) = resolve_header(&header_name, source_dir, includes) {
                    if path.exists() && visited.insert(path.clone()) {
                        if let Ok(header_content) = fs::read(&path) {
                            hasher.update(&header_content);
                            scan_and_hash_headers(
                                &header_content,
                                path.parent(),
                                includes,
                                hasher,
                                visited,
                            )?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_header_name(line: &str) -> Option<String> {
    let rest = line.strip_prefix("#include")?.trim();
    if (rest.starts_with('"') && rest.ends_with('"'))
        || (rest.starts_with('<') && rest.ends_with('>'))
    {
        Some(rest[1..rest.len() - 1].to_string())
    } else {
        None
    }
}

fn resolve_header(
    header_name: &str,
    source_dir: Option<&Path>,
    includes: &[PathBuf],
) -> Option<PathBuf> {
    if let Some(dir) = source_dir {
        let candidate = dir.join(header_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    for inc in includes {
        let candidate = inc.join(header_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn transitive_headers_invalidate_the_fingerprint() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("util.h"), "#define VALUE 1\n").unwrap();
        fs::write(dir.path().join("wrapper.h"), "#include \"util.h\"\n").unwrap();
        let source = dir.path().join("main.c");
        fs::write(
            &source,
            "#include \"wrapper.h\"\nint main() { return VALUE; }\n",
        )
        .unwrap();

        let includes = vec![dir.path().to_path_buf()];
        let before = compute_source_fingerprint(&source, &includes, &[], "gcc 13").unwrap();

        fs::write(dir.path().join("util.h"), "#define VALUE 2\n").unwrap();
        let after = compute_source_fingerprint(&source, &includes, &[], "gcc 13").unwrap();

        assert_ne!(
            before, after,
            "a transitive header change must change the fingerprint"
        );
    }

    #[test]
    fn unchanged_inputs_keep_the_fingerprint_stable() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("main.c"), "int main() { return 0; }\n").unwrap();
        let source = dir.path().join("main.c");

        let a = compute_source_fingerprint(&source, &[], &["-O2".to_string()], "gcc 13").unwrap();
        let b = compute_source_fingerprint(&source, &[], &["-O2".to_string()], "gcc 13").unwrap();
        assert_eq!(a, b);
    }
}
