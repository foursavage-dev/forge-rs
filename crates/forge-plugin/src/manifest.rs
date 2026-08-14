use std::fs;
use std::path::Path;

use crate::rule::PluginRulesManifest;

impl PluginRulesManifest {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn discover_or_load(root: &Path) -> Result<Self, String> {
        for candidate in &["Forgefile.json", "forge.rules.json", "forge.plugin.json"] {
            let p = root.join(candidate);
            if p.exists() {
                return Self::from_file(&p);
            }
        }
        Err("no custom rules manifest (Forgefile.json, forge.rules.json) found".to_string())
    }
}
