#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

/// Parses a GCC/Clang `-MMD -MF` dependency file (`.d`). Returns the
/// dependency paths listed, or `None` when the file cannot be read or the
/// target line is missing (e.g. a stale/empty depfile).
pub fn read_depfile(path: &Path) -> Option<Vec<PathBuf>> {
    let content = fs::read_to_string(path).ok()?;

    let mut deps = Vec::new();
    let mut logical = String::new();
    let mut saw_target = false;

    for raw in content.lines() {
        let line = raw.trim_end();
        if line.is_empty() {
            continue;
        }

        let continued = line.ends_with('\\');
        let piece = if continued { &line[..line.len() - 1] } else { line };
        if !piece.trim().is_empty() {
            logical.push_str(piece);
            logical.push(' ');
        }
        if continued {
            continue;
        }

        let (target, deps_part) = logical.split_once(':').unwrap_or(("", logical.as_str()));
        if !target.trim().is_empty() {
            saw_target = true;
        }
        for dep in parse_deps(deps_part) {
            deps.push(dep);
        }
        logical.clear();
    }

    if saw_target {
        Some(deps)
    } else {
        None
    }
}

/// Splits the dependency part of a `.d` line into paths, honouring the
/// backslash escape GCC/Clang emit for spaces inside a path.
fn parse_deps(after_colon: &str) -> Vec<PathBuf> {
    let mut deps = Vec::new();
    let mut token = String::new();
    let mut chars = after_colon.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(&next) = chars.peek() {
                    token.push(next);
                    chars.next();
                }
            }
            c if c.is_whitespace() => {
                if !token.is_empty() {
                    deps.push(PathBuf::from(token.clone()));
                    token.clear();
                }
            }
            _ => token.push(c),
        }
    }
    if !token.is_empty() {
        deps.push(PathBuf::from(token));
    }
    deps
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parses_single_line_depfile() {
        let dir = tempdir().unwrap();
        let depfile = dir.path().join("main.d");
        fs::write(
            &depfile,
            "build/main.o: src/main.c include/util.h include/wrapper.h\n",
        )
        .unwrap();

        let deps = read_depfile(&depfile).expect("depfile parses");
        assert_eq!(
            deps,
            vec![
                PathBuf::from("src/main.c"),
                PathBuf::from("include/util.h"),
                PathBuf::from("include/wrapper.h"),
            ]
        );
    }

    #[test]
    fn parses_continuation_lines_and_escaped_spaces() {
        let dir = tempdir().unwrap();
        let depfile = dir.path().join("util.d");
        fs::write(
            &depfile,
            "util.o: src/util.c include/my\\ header.h \\\n  include/a.h include/b.h\n",
        )
        .unwrap();

        let deps = read_depfile(&depfile).expect("depfile parses");
        assert_eq!(
            deps,
            vec![
                PathBuf::from("src/util.c"),
                PathBuf::from("include/my header.h"),
                PathBuf::from("include/a.h"),
                PathBuf::from("include/b.h"),
            ]
        );
    }

    #[test]
    fn returns_none_for_unreadable_or_blank_depfile() {
        let dir = tempdir().unwrap();
        assert!(read_depfile(&dir.path().join("missing.d")).is_none());

        let blank = dir.path().join("blank.d");
        fs::write(&blank, "  \n").unwrap();
        assert!(read_depfile(&blank).is_none());
    }
}