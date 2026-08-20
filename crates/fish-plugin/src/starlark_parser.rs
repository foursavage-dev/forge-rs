use std::collections::HashMap;
use std::path::Path;

use crate::PluginError;
use crate::rule::{PluginRulesManifest, RuleSpec};

pub struct StarlarkRulesParser;

impl StarlarkRulesParser {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<PluginRulesManifest, PluginError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::parse_str(&content)
    }

    pub fn parse_str(content: &str) -> Result<PluginRulesManifest, PluginError> {
        let mut rules = Vec::new();
        let mut current_rule: Option<RuleBuilder> = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with("fish_rule(")
                || trimmed.starts_with("rule(")
                || trimmed.starts_with("genrule(")
                || trimmed.starts_with("cc_binary(")
                || trimmed.starts_with("rust_binary(")
            {
                if let Some(builder) = current_rule.take() {
                    rules.push(builder.build()?);
                }
                current_rule = Some(RuleBuilder::default());
                continue;
            }

            if trimmed == ")" || trimmed == ")," {
                if let Some(builder) = current_rule.take() {
                    rules.push(builder.build()?);
                }
                continue;
            }

            if let (Some(builder), Some((key, val))) =
                (current_rule.as_mut(), parse_key_value(trimmed))
            {
                builder.set_field(&key, &val)?;
            }
        }

        if let Some(builder) = current_rule.take() {
            rules.push(builder.build()?);
        }

        Ok(PluginRulesManifest {
            name: "starlark-manifest".to_string(),
            rules,
        })
    }
}

#[derive(Default)]
struct RuleBuilder {
    name: Option<String>,
    command: Option<String>,
    args: Vec<String>,
    inputs: Vec<String>,
    outputs: Vec<String>,
    depends_on: Vec<String>,
    env: HashMap<String, String>,
}

impl RuleBuilder {
    fn set_field(&mut self, key: &str, val: &str) -> Result<(), PluginError> {
        match key {
            "name" => {
                self.name = Some(clean_string(val));
            }
            "cmd" | "command" | "executable" => {
                self.command = Some(clean_string(val));
            }
            "args" | "arguments" => {
                self.args = parse_string_list(val);
            }
            "srcs" | "inputs" | "sources" => {
                self.inputs = parse_string_list(val);
            }
            "outs" | "outputs" => {
                self.outputs = parse_string_list(val);
            }
            "deps" | "depends_on" => {
                self.depends_on = parse_string_list(val);
            }
            _ => {}
        }
        Ok(())
    }

    fn build(self) -> Result<RuleSpec, PluginError> {
        let name = self
            .name
            .ok_or_else(|| PluginError::Manifest("Missing required `name` in rule".to_string()))?;
        let command = self.command.unwrap_or_else(|| "echo".to_string());

        Ok(RuleSpec {
            name,
            command,
            args: self.args,
            inputs: self.inputs,
            outputs: self.outputs,
            depends_on: self.depends_on,
            env: self.env,
        })
    }
}

fn parse_key_value(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() == 2 {
        let key = parts[0].trim().to_string();
        let mut val = parts[1].trim();
        if val.ends_with(',') {
            val = val[..val.len() - 1].trim();
        }
        return Some((key, val.to_string()));
    }
    None
}

fn clean_string(val: &str) -> String {
    let trimmed = val.trim();
    let is_quoted = (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''));
    if is_quoted && trimmed.len() >= 2 {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed.to_string()
}

fn parse_string_list(val: &str) -> Vec<String> {
    let mut trimmed = val.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        trimmed = &trimmed[1..trimmed.len() - 1];
    }
    trimmed
        .split(',')
        .map(clean_string)
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_starlark_rules_content() {
        let starlark_code = r#"
fish_rule(
    name = "codegen",
    command = "python",
    args = ["tools/gen.py", "--out", "gen/code.rs"],
    inputs = ["tools/gen.py", "schema.json"],
    outputs = ["gen/code.rs"],
    deps = [":init"],
)

genrule(
    name = "compress_assets",
    command = "tar",
    args = ["-czf", "assets.tar.gz", "static/"],
    inputs = ["static/*"],
    outputs = ["assets.tar.gz"],
)
"#;

        let manifest = StarlarkRulesParser::parse_str(starlark_code).unwrap();
        assert_eq!(manifest.rules.len(), 2);

        let r1 = &manifest.rules[0];
        assert_eq!(r1.name, "codegen");
        assert_eq!(r1.command, "python");
        assert_eq!(r1.args, vec!["tools/gen.py", "--out", "gen/code.rs"]);
        assert_eq!(r1.inputs, vec!["tools/gen.py", "schema.json"]);
        assert_eq!(r1.outputs, vec!["gen/code.rs"]);
        assert_eq!(r1.depends_on, vec![":init"]);

        let r2 = &manifest.rules[1];
        assert_eq!(r2.name, "compress_assets");
        assert_eq!(r2.command, "tar");
    }
}
