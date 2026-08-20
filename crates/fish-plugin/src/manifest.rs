use std::fs;
use std::path::Path;

use crate::rule::PluginRulesManifest;

impl PluginRulesManifest {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn discover_or_load(root: &Path) -> Result<Self, String> {
        for candidate in &[
            "fishfile.json",
            "Fishfile.json",
            "fish.rules.json",
            "fish.plugin.json",
            "BUILD.fish",
            "BUILD.forge",
            "BUILD.bazel",
            "forge.rules.json",
        ] {
            let p = root.join(candidate);
            if p.exists() {
                if candidate.starts_with("BUILD.") {
                    return crate::starlark_parser::StarlarkRulesParser::parse_file(&p)
                        .map_err(|e| e.to_string());
                }
                return Self::from_file(&p);
            }
        }
        Err(
            "no custom rules manifest (fishfile.json, fish.rules.json, BUILD.fish) found"
                .to_string(),
        )
    }
}
