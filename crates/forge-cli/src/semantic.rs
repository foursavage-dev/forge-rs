#![allow(dead_code)]

use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLanguage {
    Rust,
    C,
    Cpp,
    Go,
    TypeScript,
    JavaScript,
    Python,
    Zig,
    Unknown,
}

impl SourceLanguage {
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => Self::Rust,
            Some("c") | Some("h") => Self::C,
            Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") | Some("hxx") => Self::Cpp,
            Some("go") => Self::Go,
            Some("ts") | Some("tsx") => Self::TypeScript,
            Some("js") | Some("jsx") | Some("mjs") => Self::JavaScript,
            Some("py") => Self::Python,
            Some("zig") => Self::Zig,
            _ => Self::Unknown,
        }
    }
}

pub struct SemanticNormalizer;

impl SemanticNormalizer {
    pub fn strip_comments_and_whitespace(source: &str, lang: SourceLanguage) -> String {
        match lang {
            SourceLanguage::Rust
            | SourceLanguage::C
            | SourceLanguage::Cpp
            | SourceLanguage::Go
            | SourceLanguage::TypeScript
            | SourceLanguage::JavaScript
            | SourceLanguage::Zig => Self::strip_c_style(source),
            SourceLanguage::Python => Self::strip_python_style(source),
            SourceLanguage::Unknown => source.split_whitespace().collect::<Vec<_>>().join(" "),
        }
    }

    fn strip_c_style(source: &str) -> String {
        let mut output = String::with_capacity(source.len());
        let chars: Vec<char> = source.chars().collect();
        let len = chars.len();
        let mut i = 0;
        let mut in_string = false;
        let mut string_char = '"';
        let mut in_line_comment = false;
        let mut in_block_comment = false;

        while i < len {
            let ch = chars[i];
            let next_ch = if i + 1 < len { Some(chars[i + 1]) } else { None };

            if in_line_comment {
                if ch == '\n' {
                    in_line_comment = false;
                    output.push('\n');
                }
                i += 1;
                continue;
            }

            if in_block_comment {
                if ch == '*' && next_ch == Some('/') {
                    in_block_comment = false;
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }

            if in_string {
                output.push(ch);
                if ch == '\\' && i + 1 < len {
                    output.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                if ch == string_char {
                    in_string = false;
                }
                i += 1;
                continue;
            }

            if ch == '"' || ch == '\'' || ch == '`' {
                in_string = true;
                string_char = ch;
                output.push(ch);
                i += 1;
                continue;
            }

            if ch == '/' && next_ch == Some('/') {
                in_line_comment = true;
                i += 2;
                continue;
            }

            if ch == '/' && next_ch == Some('*') {
                in_block_comment = true;
                i += 2;
                continue;
            }

            output.push(ch);
            i += 1;
        }

        output
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn strip_python_style(source: &str) -> String {
        let mut output = Vec::new();
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }
            if let Some(pos) = line.find('#') {
                output.push(line[..pos].trim().to_string());
            } else {
                output.push(trimmed.to_string());
            }
        }
        output.join("\n")
    }

    pub fn compute_semantic_hash(path: &Path) -> Result<String, std::io::Error> {
        let content = fs::read_to_string(path)?;
        let lang = SourceLanguage::from_path(path);
        let normalized = Self::strip_comments_and_whitespace(&content, lang);
        let mut hasher = blake3::Hasher::new();
        hasher.update(normalized.as_bytes());
        Ok(hasher.finalize().to_hex().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_comment_and_whitespace_invariance() {
        let code_a = r#"
            fn calculate(a: i32, b: i32) -> i32 {
                // Add the two numbers
                a + b
            }
        "#;

        let code_b = r#"
            fn calculate(a: i32, b: i32) -> i32 {
                /* Different multi-line comment */
                a + b
            }
        "#;

        let norm_a = SemanticNormalizer::strip_comments_and_whitespace(code_a, SourceLanguage::Rust);
        let norm_b = SemanticNormalizer::strip_comments_and_whitespace(code_b, SourceLanguage::Rust);

        assert_eq!(norm_a, norm_b);
    }
}
