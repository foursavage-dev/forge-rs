use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default)]
pub struct WorkspaceFeatureUnification {
    pub crate_features: BTreeMap<String, BTreeSet<String>>,
    pub duplicate_crates: BTreeMap<String, Vec<String>>,
}

impl WorkspaceFeatureUnification {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_dependency_features(&mut self, crate_name: &str, features: &[String]) {
        let entry = self
            .crate_features
            .entry(crate_name.to_string())
            .or_default();
        for f in features {
            entry.insert(f.clone());
        }
    }

    pub fn record_crate_version(&mut self, crate_name: &str, version: &str) {
        let entry = self
            .duplicate_crates
            .entry(crate_name.to_string())
            .or_default();
        if !entry.contains(&version.to_string()) {
            entry.push(version.to_string());
        }
    }

    pub fn find_duplicate_versions(&self) -> BTreeMap<String, Vec<String>> {
        self.duplicate_crates
            .iter()
            .filter(|(_, versions)| versions.len() > 1)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn generate_unified_cargo_toml_dependencies(&self) -> String {
        let mut output = String::from("[dependencies]\n");
        for (k, v) in &self.crate_features {
            if v.is_empty() {
                output.push_str(&format!("{k} = \"*\"\n"));
            } else {
                let features_json: Vec<String> = v.iter().map(|f| format!("\"{f}\"")).collect();
                output.push_str(&format!(
                    "{k} = {{ version = \"*\", features = [{}] }}\n",
                    features_json.join(", ")
                ));
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_versions_detection() {
        let mut unifier = WorkspaceFeatureUnification::new();
        unifier.record_crate_version("syn", "1.0.109");
        unifier.record_crate_version("syn", "2.0.48");
        unifier.record_crate_version("serde", "1.0.195");

        let dupes = unifier.find_duplicate_versions();
        assert_eq!(dupes.len(), 1);
        assert!(dupes.contains_key("syn"));
        assert_eq!(dupes.get("syn").unwrap().len(), 2);
    }

    #[test]
    fn test_unified_features() {
        let mut unifier = WorkspaceFeatureUnification::new();
        unifier.record_dependency_features("tokio", &["rt".to_string(), "macros".to_string()]);
        unifier.record_dependency_features("tokio", &["sync".to_string()]);

        let unified_features = unifier.crate_features.get("tokio").unwrap();
        assert_eq!(unified_features.len(), 3);
        assert!(unified_features.contains("rt"));
        assert!(unified_features.contains("macros"));
        assert!(unified_features.contains("sync"));
    }
}
