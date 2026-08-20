use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiSignature {
    pub name: String,
    pub signature_text: String,
    pub is_public: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SemanticAbiExtractor;

impl SemanticAbiExtractor {
    pub fn extract_rust_public_interface(source: &str) -> String {
        let mut signatures = BTreeSet::new();
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub struct ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("pub trait ")
                || trimmed.starts_with("pub type ")
                || trimmed.starts_with("pub mod ")
                || trimmed.starts_with("pub const ")
            {
                if let Some(header) = trimmed.split('{').next() {
                    signatures.insert(header.trim().to_string());
                } else {
                    signatures.insert(trimmed.to_string());
                }
            }
        }
        signatures.into_iter().collect::<Vec<_>>().join("\n")
    }

    pub fn compute_interface_hash(source: &str) -> String {
        let interface_text = Self::extract_rust_public_interface(source);
        blake3::hash(interface_text.as_bytes()).to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_body_change_does_not_alter_interface_hash() {
        let v1 = r#"
        pub fn add(a: i32, b: i32) -> i32 {
            a + b
        }
        fn internal_helper() {
            println!("v1");
        }
        "#;

        let v2 = r#"
        pub fn add(a: i32, b: i32) -> i32 {
            let res = a + b;
            res
        }
        fn internal_helper() {
            println!("v2 changed completely");
        }
        "#;

        let h1 = SemanticAbiExtractor::compute_interface_hash(v1);
        let h2 = SemanticAbiExtractor::compute_interface_hash(v2);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_public_api_change_modifies_interface_hash() {
        let v1 = "pub fn calculate(val: u32) -> u32 { val * 2 }";
        let v2 = "pub fn calculate(val: u64) -> u64 { val * 2 }";

        let h1 = SemanticAbiExtractor::compute_interface_hash(v1);
        let h2 = SemanticAbiExtractor::compute_interface_hash(v2);
        assert_ne!(h1, h2);
    }
}
