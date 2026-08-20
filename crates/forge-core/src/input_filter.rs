use regex::Regex;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct MicroInputFilter {
    include_regexes: Vec<Regex>,
    exclude_regexes: Vec<Regex>,
}

impl MicroInputFilter {
    pub fn new(include_globs: &[&str], exclude_globs: &[&str]) -> Self {
        let include_regexes = include_globs
            .iter()
            .filter_map(|g| Self::glob_to_regex(g))
            .collect();
        let exclude_regexes = exclude_globs
            .iter()
            .filter_map(|g| Self::glob_to_regex(g))
            .collect();

        Self {
            include_regexes,
            exclude_regexes,
        }
    }

    pub fn matches(&self, relative_path: &Path) -> bool {
        let normalized = relative_path.to_string_lossy().replace('\\', "/");

        for exc in &self.exclude_regexes {
            if exc.is_match(&normalized) {
                return false;
            }
        }

        if self.include_regexes.is_empty() {
            return true;
        }

        for inc in &self.include_regexes {
            if inc.is_match(&normalized) {
                return true;
            }
        }

        false
    }

    pub fn filter_paths(&self, base_dir: &Path, paths: &[PathBuf]) -> Vec<PathBuf> {
        paths
            .iter()
            .filter(|p| {
                let rel = p.strip_prefix(base_dir).unwrap_or(p);
                self.matches(rel)
            })
            .cloned()
            .collect()
    }

    fn glob_to_regex(glob: &str) -> Option<Regex> {
        let mut regex_str = String::from("^");
        let mut chars = glob.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '*' => {
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        if chars.peek() == Some(&'/') {
                            chars.next();
                            regex_str.push_str("(?:.*/)?");
                        } else {
                            regex_str.push_str(".*");
                        }
                    } else {
                        regex_str.push_str("[^/]*");
                    }
                }
                '?' => regex_str.push_str("[^/]"),
                '.' | '(' | ')' | '+' | '|' | '^' | '$' | '[' | ']' | '{' | '}' | '\\' => {
                    regex_str.push('\\');
                    regex_str.push(c);
                }
                _ => regex_str.push(c),
            }
        }
        regex_str.push('$');
        Regex::new(&regex_str).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_micro_input_filter_includes_and_excludes() {
        let filter = MicroInputFilter::new(
            &["src/**/*.rs", "Cargo.toml"],
            &["**/*.tmp", "src/legacy/**"],
        );

        assert!(filter.matches(Path::new("src/main.rs")));
        assert!(filter.matches(Path::new("src/utils/math.rs")));
        assert!(filter.matches(Path::new("Cargo.toml")));

        assert!(!filter.matches(Path::new("README.md")));
        assert!(!filter.matches(Path::new("src/utils/math.rs.tmp")));
        assert!(!filter.matches(Path::new("src/legacy/old.rs")));
    }
}
