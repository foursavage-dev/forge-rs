use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstSubTree {
    pub symbol_name: String,
    pub kind: String,
    pub content_hash: String,
    pub byte_range: (usize, usize),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AstCacheIndex {
    pub file_trees: HashMap<String, Vec<AstSubTree>>,
}

impl AstCacheIndex {
    pub fn new() -> Self {
        Self {
            file_trees: HashMap::new(),
        }
    }

    pub fn record_file_subtrees(&mut self, file_path: &str, subtrees: Vec<AstSubTree>) {
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
            },
            AstSubTree {
                symbol_name: "format_receipt".to_string(),
                kind: "fn".to_string(),
                content_hash: "hash_v1".to_string(),
                byte_range: (101, 200),
            },
        ];

        cache.record_file_subtrees("src/lib.rs", initial);

        let updated = vec![
            AstSubTree {
                symbol_name: "calculate_tax".to_string(),
                kind: "fn".to_string(),
                content_hash: "hash_v2".to_string(),
                byte_range: (0, 120),
            },
            AstSubTree {
                symbol_name: "format_receipt".to_string(),
                kind: "fn".to_string(),
                content_hash: "hash_v1".to_string(),
                byte_range: (121, 220),
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
                },
                AstSubTree {
                    symbol_name: "removed".to_string(),
                    kind: "fn".to_string(),
                    content_hash: "h2".to_string(),
                    byte_range: (11, 20),
                },
            ],
        );

        // `removed` disappears from the new subtree list.
        let changed = cache.compute_changed_symbols(
            "src/lib.rs",
            &[AstSubTree {
                symbol_name: "keep".to_string(),
                kind: "fn".to_string(),
                content_hash: "h1".to_string(),
                byte_range: (0, 10),
            }],
        );

        assert_eq!(changed, vec!["removed".to_string()]);
    }
}
