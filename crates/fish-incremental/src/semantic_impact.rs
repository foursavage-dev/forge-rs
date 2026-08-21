use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Default)]
pub struct SemanticImpactGraph {
    symbol_to_tests: HashMap<String, BTreeSet<String>>,
}

impl SemanticImpactGraph {
    pub fn new() -> Self {
        Self {
            symbol_to_tests: HashMap::new(),
        }
    }

    pub fn map_symbol_to_test(&mut self, symbol: &str, test_target: &str) {
        self.symbol_to_tests
            .entry(symbol.to_string())
            .or_default()
            .insert(test_target.to_string());
    }

    pub fn find_impacted_tests(&self, modified_symbols: &[String]) -> Vec<String> {
        let mut impacted = BTreeSet::new();
        for sym in modified_symbols {
            if let Some(tests) = self.symbol_to_tests.get(sym) {
                for t in tests {
                    impacted.insert(t.clone());
                }
            }
        }
        impacted.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_symbol_impact_mapping() {
        let mut graph = SemanticImpactGraph::new();
        graph.map_symbol_to_test("calculate_tax", "tests::test_taxation");
        graph.map_symbol_to_test("calculate_tax", "tests::test_invoice");
        graph.map_symbol_to_test("format_user_name", "tests::test_user");

        let impacted = graph.find_impacted_tests(&["calculate_tax".to_string()]);
        assert_eq!(impacted.len(), 2);
        assert!(impacted.contains(&"tests::test_taxation".to_string()));
        assert!(impacted.contains(&"tests::test_invoice".to_string()));
    }
}
