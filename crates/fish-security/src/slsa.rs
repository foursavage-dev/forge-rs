use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaProvenance {
    #[serde(rename = "_type")]
    pub doc_type: String,
    pub predicate_type: String,
    pub subject: Vec<SlsaSubject>,
    pub predicate: SlsaPredicate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaSubject {
    pub name: String,
    pub digest: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaPredicate {
    pub builder: SlsaBuilder,
    pub build_type: String,
    pub invocation: SlsaInvocation,
    pub materials: Vec<SlsaMaterial>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaBuilder {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaInvocation {
    pub config_source: HashMap<String, String>,
    pub parameters: HashMap<String, String>,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaMaterial {
    pub uri: String,
    pub digest: HashMap<String, String>,
}

pub struct SlsaGenerator;

impl SlsaGenerator {
    pub fn generate_v1(
        artifact_name: &str,
        blake3_hash: &str,
        builder_version: &str,
    ) -> SlsaProvenance {
        let mut digests = HashMap::new();
        digests.insert("blake3".to_string(), blake3_hash.to_string());

        let subject = vec![SlsaSubject {
            name: artifact_name.to_string(),
            digest: digests,
        }];

        let mut config = HashMap::new();
        config.insert("manifest".to_string(), "fish.toml".to_string());

        SlsaProvenance {
            doc_type: "https://in-toto.io/Statement/v1".to_string(),
            predicate_type: "https://slsa.dev/provenance/v1".to_string(),
            subject,
            predicate: SlsaPredicate {
                builder: SlsaBuilder {
                    id: "https://github.com/requla11/fish".to_string(),
                    version: builder_version.to_string(),
                },
                build_type: "https://fish.build/tasks/v1".to_string(),
                invocation: SlsaInvocation {
                    config_source: config,
                    parameters: HashMap::new(),
                    environment: HashMap::new(),
                },
                materials: Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slsa_provenance_generation() {
        let doc = SlsaGenerator::generate_v1("output.bin", "abc123blake3hash", "0.2.0");
        assert_eq!(doc.doc_type, "https://in-toto.io/Statement/v1");
        assert_eq!(doc.subject.len(), 1);
        assert_eq!(doc.subject[0].name, "output.bin");
        assert_eq!(doc.predicate.builder.id, "https://github.com/requla11/fish");
    }
}
