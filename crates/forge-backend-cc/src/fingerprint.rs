use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::depfile::read_depfile;
use forge_core::FingerprintUtils;

pub fn compute_source_fingerprint(
    source_path: &Path,
    includes: &[PathBuf],
    flags: &[String],
    compiler_version: &str,
    depfile: Option<&Path>,
) -> Result<String, std::io::Error> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(compiler_version.as_bytes());
    hasher.update(b"\nflags:\n");
    for flag in flags {
        hasher.update(flag.as_bytes());
        hasher.update(b"\n");
    }

    if source_path.exists() {
        FingerprintUtils::hash_file_into(source_path, &mut hasher)?;

        if let Some(deps) = depfile.and_then(read_depfile) {
            let base = source_path.parent().unwrap_or_else(|| Path::new("."));
            for dep in deps {
                let dep = if dep.is_absolute() {
                    dep
                } else {
                    base.join(dep)
                };
                if dep == source_path {
                    continue;
                }
                if dep.exists() {
                    let _ = FingerprintUtils::hash_file_into(&dep, &mut hasher);
                }
            }
        } else if let Ok(content) = fs::read(source_path) {
            let parent = source_path.parent();
            let mut visited = HashSet::new();
            scan_and_hash_headers(&content, parent, includes, &mut hasher, &mut visited)?;
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

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
                        FingerprintUtils::hash_file_into(&path, hasher)?;
                        if let Ok(header_content) = fs::read(&path) {
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
        let before = compute_source_fingerprint(&source, &includes, &[], "gcc 13", None).unwrap();

        fs::write(dir.path().join("util.h"), "#define VALUE 2\n").unwrap();
        let after = compute_source_fingerprint(&source, &includes, &[], "gcc 13", None).unwrap();

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

        let a =
            compute_source_fingerprint(&source, &[], &["-O2".to_string()], "gcc 13", None).unwrap();
        let b =
            compute_source_fingerprint(&source, &[], &["-O2".to_string()], "gcc 13", None).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn depfile_headers_drive_the_fingerprint() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("main.c"), "int main() { return 0; }\n").unwrap();
        fs::write(dir.path().join("generated.h"), "#define GENERATED 1\n").unwrap();
        let source = dir.path().join("main.c");

        let depfile = dir.path().join("main.d");
        fs::write(&depfile, "main.o: main.c generated.h\n").unwrap();

        let before =
            compute_source_fingerprint(&source, &[], &[], "gcc 13", Some(&depfile)).unwrap();

        fs::write(dir.path().join("generated.h"), "#define GENERATED 2\n").unwrap();
        let after =
            compute_source_fingerprint(&source, &[], &[], "gcc 13", Some(&depfile)).unwrap();

        assert_ne!(
            before, after,
            "a depfile-listed header change must change the fingerprint"
        );

        let scanner_only = compute_source_fingerprint(&source, &[], &[], "gcc 13", None).unwrap();
        assert_ne!(before, scanner_only);
    }
}
