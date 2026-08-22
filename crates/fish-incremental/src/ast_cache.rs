use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    C,
    Cpp,
    Java,
    Swift,
    Dart,
    Zig,
    DotNet,
}

impl Language {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Self::Rust),
            "ts" | "tsx" => Some(Self::TypeScript),
            "js" | "jsx" | "mjs" => Some(Self::JavaScript),
            "py" => Some(Self::Python),
            "go" => Some(Self::Go),
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" => Some(Self::Cpp),
            "java" => Some(Self::Java),
            "swift" => Some(Self::Swift),
            "dart" => Some(Self::Dart),
            "zig" => Some(Self::Zig),
            "cs" => Some(Self::DotNet),
            _ => None,
        }
    }

    pub fn supports_subtree_caching(&self) -> bool {
        matches!(
            self,
            Self::Rust
                | Self::TypeScript
                | Self::JavaScript
                | Self::Python
                | Self::Go
                | Self::C
                | Self::Cpp
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstSubTree {
    pub symbol_name: String,
    pub kind: String, // fn, struct, class, impl, const, etc.
    pub content_hash: String,
    pub byte_range: (usize, usize),
    pub language: Language,
    pub dependencies: Vec<String>, // symbols this depends on
    pub complexity_score: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AstCacheIndex {
    pub file_trees: HashMap<String, Vec<AstSubTree>>,
    pub symbol_index: HashMap<String, Vec<String>>, // symbol -> files containing it
    pub language_stats: HashMap<String, LanguageStats>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageStats {
    pub file_count: usize,
    pub symbol_count: usize,
    pub total_bytes: usize,
    pub cached_symbols: usize,
}

impl AstCacheIndex {
    pub fn new() -> Self {
        Self {
            file_trees: HashMap::new(),
            symbol_index: HashMap::new(),
            language_stats: HashMap::new(),
        }
    }

    pub fn record_file_subtrees(&mut self, file_path: &str, subtrees: Vec<AstSubTree>) {
        // Update symbol index
        if let Some(old_trees) = self.file_trees.get(file_path) {
            for old in old_trees {
                if let Some(files) = self.symbol_index.get_mut(&old.symbol_name) {
                    files.retain(|f| f != file_path);
                    if files.is_empty() {
                        self.symbol_index.remove(&old.symbol_name);
                    }
                }
            }
        }

        for subtree in &subtrees {
            self.symbol_index
                .entry(subtree.symbol_name.clone())
                .or_default()
                .push(file_path.to_string());
        }

        // Update language stats
        if let Some(first) = subtrees.first() {
            let lang_key = format!("{:?}", first.language);
            let stats = self.language_stats.entry(lang_key).or_default();
            stats.file_count += 1;
            stats.symbol_count += subtrees.len();
            stats.total_bytes += subtrees.iter().map(|s| s.byte_range.1 - s.byte_range.0).sum::<usize>();
        }

        self.file_trees.insert(file_path.to_string(), subtrees);
    }

    pub fn compute_changed_symbols(
        &self,
        file_path: &str,
        new_subtrees: &[AstSubTree],
    ) -> Vec<String> {
        let mut changed = Vec::new();
        let old_trees = match self.file_trees.get(file_path) {
            Some(t) => t,
            None => return new_subtrees.iter().map(|s| s.symbol_name.clone()).collect(),
        };

        let mut old_map = HashMap::with_capacity(old_trees.len());
        for s in old_trees {
            old_map.insert(s.symbol_name.as_str(), s.content_hash.as_str());
        }

        for s in new_subtrees {
            match old_map.get(s.symbol_name.as_str()) {
                Some(old_hash) if *old_hash == s.content_hash.as_str() => {}
                _ => changed.push(s.symbol_name.clone()),
            }
        }

        let mut new_names = std::collections::HashSet::with_capacity(new_subtrees.len());
        for s in new_subtrees {
            new_names.insert(s.symbol_name.as_str());
        }
        for s in old_trees {
            if !new_names.contains(s.symbol_name.as_str()) {
                changed.push(s.symbol_name.clone());
            }
        }

        changed
    }

    /// Compute transitive impact - which symbols are affected by changed symbols
    pub fn compute_transitive_impact(&self, changed_symbols: &[String]) -> Vec<String> {
        let mut impacted = std::collections::HashSet::new();
        let mut queue: Vec<String> = changed_symbols.to_vec();
        let mut visited = std::collections::HashSet::new();

        while let Some(symbol) = queue.pop() {
            if visited.contains(&symbol) {
                continue;
            }
            visited.insert(symbol.clone());

            // Find all files containing this symbol
            for (file_path, subtrees) in &self.file_trees {
                for subtree in subtrees {
                    if subtree.dependencies.contains(&symbol) {
                        if impacted.insert(subtree.symbol_name.clone()) {
                            queue.push(subtree.symbol_name.clone());
                        }
                    }
                }
            }
        }

        impacted.into_iter().collect()
    }

    /// Parse file content and extract subtrees (cross-language)
    pub fn parse_file_content(file_path: &str, content: &str) -> Vec<AstSubTree> {
        let ext = std::path::Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        let language = Language::from_extension(ext).unwrap_or(Language::Rust);

        match language {
            Language::Rust => Self::parse_rust_content(content),
            Language::TypeScript | Language::JavaScript => Self::parse_ts_content(content),
            Language::Python => Self::parse_python_content(content),
            Language::Go => Self::parse_go_content(content),
            _ => Self::parse_generic_content(content, language),
        }
    }

    fn parse_rust_content(content: &str) -> Vec<AstSubTree> {
        let mut subtrees = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut byte_offset = 0;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("async fn ") {
                if let Some(name) = Self::extract_rust_fn_name(trimmed) {
                    let end = Self::find_rust_block_end(&lines, idx);
                    let hash = blake3::hash(trimmed.as_bytes()).to_hex().to_string();
                    subtrees.push(AstSubTree {
                        symbol_name: name,
                        kind: "fn".to_string(),
                        content_hash: hash,
                        byte_range: (byte_offset, byte_offset + line.len()),
                        language: Language::Rust,
                        dependencies: Self::extract_dependencies(trimmed),
                        complexity_score: Self::calculate_complexity(trimmed),
                    });
                    let _ = end;
                }
            } else if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
                if let Some(name) = Self::extract_rust_struct_name(trimmed) {
                    let hash = blake3::hash(trimmed.as_bytes()).to_hex().to_string();
                    subtrees.push(AstSubTree {
                        symbol_name: name,
                        kind: "struct".to_string(),
                        content_hash: hash,
                        byte_range: (byte_offset, byte_offset + line.len()),
                        language: Language::Rust,
                        dependencies: Vec::new(),
                        complexity_score: 1,
                    });
                }
            } else if trimmed.starts_with("impl ") {
                let name = trimmed.trim_start_matches("impl ").split_whitespace().next().unwrap_or("unknown").to_string();
                let hash = blake3::hash(trimmed.as_bytes()).to_hex().to_string();
                subtrees.push(AstSubTree {
                    symbol_name: format!("impl_{}", name),
                    kind: "impl".to_string(),
                    content_hash: hash,
                    byte_range: (byte_offset, byte_offset + line.len()),
                    language: Language::Rust,
                    dependencies: vec![name],
                    complexity_score: 2,
                });
            }

            byte_offset += line.len() + 1; // +1 for newline
        }

        subtrees
    }

    fn parse_ts_content(content: &str) -> Vec<AstSubTree> {
        let mut subtrees = Vec::new();
        let mut byte_offset = 0;

        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("function ") || trimmed.starts_with("export function ") || trimmed.starts_with("async function ") {
                if let Some(name) = Self::extract_js_fn_name(trimmed) {
                    let hash = blake3::hash(trimmed.as_bytes()).to_hex().to_string();
                    subtrees.push(AstSubTree {
                        symbol_name: name,
                        kind: "function".to_string(),
                        content_hash: hash,
                        byte_range: (byte_offset, byte_offset + line.len()),
                        language: Language::TypeScript,
                        dependencies: Self::extract_dependencies(trimmed),
                        complexity_score: Self::calculate_complexity(trimmed),
                    });
                }
            } else if trimmed.starts_with("class ") || trimmed.starts_with("export class ") {
                if let Some(name) = Self::extract_class_name(trimmed) {
                    let hash = blake3::hash(trimmed.as_bytes()).to_hex().to_string();
                    subtrees.push(AstSubTree {
                        symbol_name: name,
                        kind: "class".to_string(),
                        content_hash: hash,
                        byte_range: (byte_offset, byte_offset + line.len()),
                        language: Language::TypeScript,
                        dependencies: Vec::new(),
                        complexity_score: 3,
                    });
                }
            } else if trimmed.contains("=>") && (trimmed.contains("const ") || trimmed.contains("let ")) {
                // Arrow function
                if let Some(name) = Self::extract_variable_name(trimmed) {
                    let hash = blake3::hash(trimmed.as_bytes()).to_hex().to_string();
                    subtrees.push(AstSubTree {
                        symbol_name: name,
                        kind: "arrow_fn".to_string(),
                        content_hash: hash,
                        byte_range: (byte_offset, byte_offset + line.len()),
                        language: Language::TypeScript,
                        dependencies: Self::extract_dependencies(trimmed),
                        complexity_score: 1,
                    });
                }
            }

            byte_offset += line.len() + 1;
        }

        subtrees
    }

    fn parse_python_content(content: &str) -> Vec<AstSubTree> {
        let mut subtrees = Vec::new();
        let mut byte_offset = 0;

        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("def ") {
                if let Some(name) = Self::extract_python_fn_name(trimmed) {
                    let hash = blake3::hash(trimmed.as_bytes()).to_hex().to_string();
                    subtrees.push(AstSubTree {
                        symbol_name: name,
                        kind: "function".to_string(),
                        content_hash: hash,
                        byte_range: (byte_offset, byte_offset + line.len()),
                        language: Language::Python,
                        dependencies: Self::extract_dependencies(trimmed),
                        complexity_score: Self::calculate_complexity(trimmed),
                    });
                }
            } else if trimmed.starts_with("class ") {
                if let Some(name) = Self::extract_class_name(trimmed) {
                    let hash = blake3::hash(trimmed.as_bytes()).to_hex().to_string();
                    subtrees.push(AstSubTree {
                        symbol_name: name,
                        kind: "class".to_string(),
                        content_hash: hash,
                        byte_range: (byte_offset, byte_offset + line.len()),
                        language: Language::Python,
                        dependencies: Vec::new(),
                        complexity_score: 3,
                    });
                }
            }

            byte_offset += line.len() + 1;
        }

        subtrees
    }

    fn parse_go_content(content: &str) -> Vec<AstSubTree> {
        let mut subtrees = Vec::new();
        let mut byte_offset = 0;

        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("func ") {
                if let Some(name) = Self::extract_go_fn_name(trimmed) {
                    let hash = blake3::hash(trimmed.as_bytes()).to_hex().to_string();
                    subtrees.push(AstSubTree {
                        symbol_name: name,
                        kind: "func".to_string(),
                        content_hash: hash,
                        byte_range: (byte_offset, byte_offset + line.len()),
                        language: Language::Go,
                        dependencies: Self::extract_dependencies(trimmed),
                        complexity_score: Self::calculate_complexity(trimmed),
                    });
                }
            } else if trimmed.starts_with("type ") && trimmed.contains("struct") {
                if let Some(name) = Self::extract_go_type_name(trimmed) {
                    let hash = blake3::hash(trimmed.as_bytes()).to_hex().to_string();
                    subtrees.push(AstSubTree {
                        symbol_name: name,
                        kind: "struct".to_string(),
                        content_hash: hash,
                        byte_range: (byte_offset, byte_offset + line.len()),
                        language: Language::Go,
                        dependencies: Vec::new(),
                        complexity_score: 2,
                    });
                }
            }

            byte_offset += line.len() + 1;
        }

        subtrees
    }

    fn parse_generic_content(content: &str, language: Language) -> Vec<AstSubTree> {
        // Fallback: treat each non-empty line as potential symbol
        let mut subtrees = Vec::new();
        let mut byte_offset = 0;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#") {
                byte_offset += line.len() + 1;
                continue;
            }

            if trimmed.len() > 10 {
                let hash = blake3::hash(trimmed.as_bytes()).to_hex().to_string();
                subtrees.push(AstSubTree {
                    symbol_name: format!("symbol_{}", byte_offset),
                    kind: "generic".to_string(),
                    content_hash: hash,
                    byte_range: (byte_offset, byte_offset + line.len()),
                    language: language.clone(),
                    dependencies: Vec::new(),
                    complexity_score: 1,
                });
            }

            byte_offset += line.len() + 1;
        }

        subtrees
    }

    // Helper extractors
    fn extract_rust_fn_name(line: &str) -> Option<String> {
        let line = line.trim_start_matches("pub ").trim_start_matches("async ");
        if line.starts_with("fn ") {
            let rest = &line[3..];
            let name = rest.split(|c: char| c == '(' || c == '<' || c.is_whitespace()).next()?;
            Some(name.to_string())
        } else {
            None
        }
    }

    fn extract_rust_struct_name(line: &str) -> Option<String> {
        let line = line.trim_start_matches("pub ");
        if line.starts_with("struct ") {
            let rest = &line[7..];
            let name = rest.split(|c: char| c == '<' || c == '{' || c.is_whitespace()).next()?;
            Some(name.to_string())
        } else {
            None
        }
    }

    fn extract_js_fn_name(line: &str) -> Option<String> {
        let line = line.trim_start_matches("export ").trim_start_matches("async ");
        if line.starts_with("function ") {
            let rest = &line[9..];
            let name = rest.split(|c: char| c == '(' || c.is_whitespace()).next()?;
            Some(name.to_string())
        } else {
            None
        }
    }

    fn extract_class_name(line: &str) -> Option<String> {
        let line = line.trim_start_matches("export ");
        if line.starts_with("class ") {
            let rest = &line[6..];
            let name = rest.split(|c: char| c == ' ' || c == '{' || c == '<' || c == '(').next()?;
            Some(name.to_string())
        } else {
            None
        }
    }

    fn extract_variable_name(line: &str) -> Option<String> {
        if let Some(eq_pos) = line.find('=') {
            let before_eq = &line[..eq_pos];
            let parts: Vec<&str> = before_eq.split_whitespace().collect();
            if let Some(last) = parts.last() {
                return Some(last.trim_matches(|c: char| c == ';' || c == ':').to_string());
            }
        }
        None
    }

    fn extract_python_fn_name(line: &str) -> Option<String> {
        if line.starts_with("def ") {
            let rest = &line[4..];
            let name = rest.split(|c: char| c == '(' || c == ':').next()?;
            Some(name.trim().to_string())
        } else {
            None
        }
    }

    fn extract_go_fn_name(line: &str) -> Option<String> {
        if line.starts_with("func ") {
            let rest = &line[5..];
            // Handle method: func (r Receiver) Name(
            let rest = if rest.starts_with('(') {
                if let Some(close) = rest.find(") ") {
                    &rest[close + 2..]
                } else {
                    rest
                }
            } else {
                rest
            };
            let name = rest.split(|c: char| c == '(' || c.is_whitespace()).next()?;
            Some(name.to_string())
        } else {
            None
        }
    }

    fn extract_go_type_name(line: &str) -> Option<String> {
        if line.starts_with("type ") {
            let rest = &line[5..];
            let name = rest.split_whitespace().next()?;
            Some(name.to_string())
        } else {
            None
        }
    }

    fn extract_dependencies(line: &str) -> Vec<String> {
        // Simple heuristic: find capitalized words or function calls
        let mut deps = Vec::new();
        for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
            if word.len() > 3 && word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                deps.push(word.to_string());
            }
        }
        deps
    }

    fn calculate_complexity(line: &str) -> u32 {
        let mut score = 1;
        score += line.matches("if ").count() as u32;
        score += line.matches("for ").count() as u32;
        score += line.matches("while ").count() as u32;
        score += line.matches("match ").count() as u32;
        score += line.matches("&&").count() as u32;
        score += line.matches("||").count() as u32;
        score
    }

    fn find_rust_block_end(lines: &[&str], start_idx: usize) -> usize {
        let mut brace_count = 0;
        let mut started = false;
        
        for i in start_idx..lines.len() {
            for ch in lines[i].chars() {
                if ch == '{' {
                    brace_count += 1;
                    started = true;
                } else if ch == '}' {
                    brace_count -= 1;
                    if started && brace_count == 0 {
                        return i;
                    }
                }
            }
        }
        
        start_idx
    }

    pub fn get_stats(&self) -> HashMap<String, LanguageStats> {
        self.language_stats.clone()
    }

    pub fn total_symbols(&self) -> usize {
        self.file_trees.values().map(|v| v.len()).sum()
    }

    pub fn get_file_symbols(&self, file_path: &str) -> Option<&Vec<AstSubTree>> {
        self.file_trees.get(file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_subtree_caching() {
        let mut cache = AstCacheIndex::new();
        let initial = vec![
            AstSubTree {
                symbol_name: "calculate_tax".to_string(),
                kind: "fn".to_string(),
                content_hash: "hash_v1".to_string(),
                byte_range: (0, 100),
                language: Language::Rust,
                dependencies: vec![],
                complexity_score: 1,
            },
            AstSubTree {
                symbol_name: "format_receipt".to_string(),
                kind: "fn".to_string(),
                content_hash: "hash_v1".to_string(),
                byte_range: (101, 200),
                language: Language::Rust,
                dependencies: vec![],
                complexity_score: 1,
            },
        ];

        cache.record_file_subtrees("src/lib.rs", initial);

        let updated = vec![
            AstSubTree {
                symbol_name: "calculate_tax".to_string(),
                kind: "fn".to_string(),
                content_hash: "hash_v2".to_string(),
                byte_range: (0, 120),
                language: Language::Rust,
                dependencies: vec![],
                complexity_score: 2,
            },
            AstSubTree {
                symbol_name: "format_receipt".to_string(),
                kind: "fn".to_string(),
                content_hash: "hash_v1".to_string(),
                byte_range: (121, 220),
                language: Language::Rust,
                dependencies: vec![],
                complexity_score: 1,
            },
        ];

        let changed = cache.compute_changed_symbols("src/lib.rs", &updated);
        assert_eq!(changed, vec!["calculate_tax".to_string()]);
    }

    #[test]
    fn removed_symbols_are_reported_as_changed() {
        let mut cache = AstCacheIndex::new();
        cache.record_file_subtrees(
            "src/lib.rs",
            vec![
                AstSubTree {
                    symbol_name: "keep".to_string(),
                    kind: "fn".to_string(),
                    content_hash: "h1".to_string(),
                    byte_range: (0, 10),
                    language: Language::Rust,
                    dependencies: vec![],
                    complexity_score: 1,
                },
                AstSubTree {
                    symbol_name: "removed".to_string(),
                    kind: "fn".to_string(),
                    content_hash: "h2".to_string(),
                    byte_range: (11, 20),
                    language: Language::Rust,
                    dependencies: vec![],
                    complexity_score: 1,
                },
            ],
        );

        let changed = cache.compute_changed_symbols(
            "src/lib.rs",
            &[AstSubTree {
                symbol_name: "keep".to_string(),
                kind: "fn".to_string(),
                content_hash: "h1".to_string(),
                byte_range: (0, 10),
                language: Language::Rust,
                dependencies: vec![],
                complexity_score: 1,
            }],
        );

        assert_eq!(changed, vec!["removed".to_string()]);
    }

    #[test]
    fn test_cross_language_parsing() {
        let rust_code = r#"
            pub fn calculate_tax(amount: f64) -> f64 {
                amount * 0.2
            }
            pub struct Receipt {
                total: f64
            }
        "#;

        let subtrees = AstCacheIndex::parse_file_content("src/lib.rs", rust_code);
        assert!(subtrees.iter().any(|s| s.symbol_name == "calculate_tax"));
        assert!(subtrees.iter().any(|s| s.symbol_name == "Receipt"));

        let ts_code = r#"
            export function formatReceipt(total: number): string {
                return `$${total}`;
            }
            export class TaxCalculator {
            }
        "#;

        let ts_subtrees = AstCacheIndex::parse_file_content("src/index.ts", ts_code);
        assert!(ts_subtrees.iter().any(|s| s.symbol_name == "formatReceipt"));
        assert!(ts_subtrees.iter().any(|s| s.symbol_name == "TaxCalculator"));

        let py_code = r#"
            def calculate_tax(amount):
                return amount * 0.2
            
            class Receipt:
                pass
        "#;

        let py_subtrees = AstCacheIndex::parse_file_content("src/main.py", py_code);
        assert!(py_subtrees.iter().any(|s| s.symbol_name == "calculate_tax"));
        assert!(py_subtrees.iter().any(|s| s.symbol_name == "Receipt"));

        let go_code = r#"
            func CalculateTax(amount float64) float64 {
                return amount * 0.2
            }
            type Receipt struct {
                Total float64
            }
        "#;

        let go_subtrees = AstCacheIndex::parse_file_content("main.go", go_code);
        assert!(go_subtrees.iter().any(|s| s.symbol_name == "CalculateTax"));
    }

    #[test]
    fn test_transitive_impact() {
        let mut cache = AstCacheIndex::new();
        
        cache.record_file_subtrees("src/lib.rs", vec![
            AstSubTree {
                symbol_name: "calculate_tax".to_string(),
                kind: "fn".to_string(),
                content_hash: "h1".to_string(),
                byte_range: (0, 10),
                language: Language::Rust,
                dependencies: vec![],
                complexity_score: 1,
            },
            AstSubTree {
                symbol_name: "format_receipt".to_string(),
                kind: "fn".to_string(),
                content_hash: "h2".to_string(),
                byte_range: (11, 20),
                language: Language::Rust,
                dependencies: vec!["calculate_tax".to_string()],
                complexity_score: 1,
            },
            AstSubTree {
                symbol_name: "print_receipt".to_string(),
                kind: "fn".to_string(),
                content_hash: "h3".to_string(),
                byte_range: (21, 30),
                language: Language::Rust,
                dependencies: vec!["format_receipt".to_string()],
                complexity_score: 1,
            },
        ]);

        let impacted = cache.compute_transitive_impact(&["calculate_tax".to_string()]);
        assert!(impacted.contains(&"format_receipt".to_string()));
        assert!(impacted.contains(&"print_receipt".to_string()));
    }
}
