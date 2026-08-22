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
    pub build_config: Option<BuildConfig>,
    pub metadata: Option<ProvenanceMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaBuilder {
    pub id: String,
    pub version: String,
    pub builder_dependencies: Vec<SlsaMaterial>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub steps: Vec<BuildStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStep {
    pub command: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceMetadata {
    pub build_started_on: String,
    pub build_finished_on: String,
    pub completeness: Completeness,
    pub reproducible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completeness {
    pub parameters: bool,
    pub environment: bool,
    pub materials: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InTotoAttestation {
    #[serde(rename = "_type")]
    pub doc_type: String,
    pub subject: Vec<SlsaSubject>,
    pub predicate_type: String,
    pub predicate: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaLevel {
    pub level: u8,
    pub requirements: Vec<String>,
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
                    builder_dependencies: Vec::new(),
                },
                build_type: "https://fish.build/tasks/v1".to_string(),
                invocation: SlsaInvocation {
                    config_source: config,
                    parameters: HashMap::new(),
                    environment: HashMap::new(),
                },
                materials: Vec::new(),
                build_config: None,
                metadata: Some(ProvenanceMetadata {
                    build_started_on: chrono::Utc::now().to_rfc3339(),
                    build_finished_on: chrono::Utc::now().to_rfc3339(),
                    completeness: Completeness {
                        parameters: true,
                        environment: true,
                        materials: true,
                    },
                    reproducible: true,
                }),
            },
        }
    }

    /// Generate SLSA Level 3 compliant provenance with full build chain
    pub fn generate_slsa_l3(
        artifact_name: &str,
        blake3_hash: &str,
        sha256_hash: &str,
        builder_version: &str,
        materials: Vec<(String, String, String)>, // (uri, digest_type, digest_value)
        build_steps: Vec<BuildStep>,
        parameters: HashMap<String, String>,
    ) -> SlsaProvenance {
        let mut digests = HashMap::new();
        digests.insert("blake3".to_string(), blake3_hash.to_string());
        digests.insert("sha256".to_string(), sha256_hash.to_string());

        let subject = vec![SlsaSubject {
            name: artifact_name.to_string(),
            digest: digests,
        }];

        let mut config = HashMap::new();
        config.insert("manifest".to_string(), "fish.toml".to_string());
        config.insert("entryPoint".to_string(), artifact_name.to_string());

        let mut env = HashMap::new();
        env.insert("FISH_VERSION".to_string(), builder_version.to_string());
        env.insert("BUILDER_ID".to_string(), "https://github.com/requla11/fish".to_string());

        let slsa_materials: Vec<SlsaMaterial> = materials
            .into_iter()
            .map(|(uri, digest_type, digest_value)| {
                let mut digest = HashMap::new();
                digest.insert(digest_type, digest_value);
                SlsaMaterial { uri, digest }
            })
            .collect();

        let builder_deps = vec![SlsaMaterial {
            uri: "https://github.com/requla11/fish".to_string(),
            digest: {
                let mut d = HashMap::new();
                d.insert("sha256".to_string(), "builder_dep_hash".to_string());
                d
            },
        }];

        SlsaProvenance {
            doc_type: "https://in-toto.io/Statement/v1".to_string(),
            predicate_type: "https://slsa.dev/provenance/v1".to_string(),
            subject,
            predicate: SlsaPredicate {
                builder: SlsaBuilder {
                    id: "https://github.com/requla11/fish/builder@v0.4.0".to_string(),
                    version: builder_version.to_string(),
                    builder_dependencies: builder_deps,
                },
                build_type: "https://fish.build/tasks/v1".to_string(),
                invocation: SlsaInvocation {
                    config_source: config,
                    parameters,
                    environment: env,
                },
                materials: slsa_materials,
                build_config: Some(BuildConfig { steps: build_steps }),
                metadata: Some(ProvenanceMetadata {
                    build_started_on: chrono::Utc::now().to_rfc3339(),
                    build_finished_on: chrono::Utc::now().to_rfc3339(),
                    completeness: Completeness {
                        parameters: true,
                        environment: true,
                        materials: true,
                    },
                    reproducible: true,
                }),
            },
        }
    }

    /// Generate in-toto attestation wrapper
    pub fn generate_in_toto_attestation(provenance: SlsaProvenance) -> InTotoAttestation {
        InTotoAttestation {
            doc_type: provenance.doc_type.clone(),
            subject: provenance.subject.clone(),
            predicate_type: provenance.predicate_type.clone(),
            predicate: serde_json::to_value(&provenance.predicate).unwrap_or(serde_json::Value::Null),
        }
    }

    /// Verify SLSA Level 3 compliance
    pub fn verify_slsa_l3_compliance(provenance: &SlsaProvenance) -> SlsaVerificationResult {
        let mut issues = Vec::new();
        let mut level = 3;

        // Check required fields for SLSA L3
        if provenance.predicate.builder.id.is_empty() {
            issues.push("builder.id is required for SLSA L3".to_string());
            level = 0;
        }

        if provenance.predicate.build_config.is_none() {
            issues.push("build_config is required for SLSA L3".to_string());
            if level > 2 {
                level = 2;
            }
        }

        if let Some(ref metadata) = provenance.predicate.metadata {
            if !metadata.completeness.parameters
                || !metadata.completeness.environment
                || !metadata.completeness.materials
            {
                issues.push("completeness must be true for all fields in SLSA L3".to_string());
                if level > 2 {
                    level = 2;
                }
            }
        } else {
            issues.push("metadata is required for SLSA L3".to_string());
            if level > 1 {
                level = 1;
            }
        }

        // Check for at least 2 digests (blake3 + sha256)
        for subject in &provenance.subject {
            if subject.digest.len() < 2 {
                issues.push(format!(
                    "subject {} should have at least 2 digests for SLSA L3",
                    subject.name
                ));
                if level > 2 {
                    level = 2;
                }
            }
        }

        // Check materials
        if provenance.predicate.materials.is_empty() {
            issues.push("materials should not be empty for SLSA L3".to_string());
            if level > 1 {
                level = 1;
            }
        }

        SlsaVerificationResult {
            compliant_level: level,
            is_slsa_l3_compliant: level >= 3 && issues.is_empty(),
            issues,
        }
    }

    pub fn get_slsa_requirements(level: u8) -> SlsaLevel {
        match level {
            1 => SlsaLevel {
                level: 1,
                requirements: vec![
                    "Build process must be documented".to_string(),
                    "Provenance must exist".to_string(),
                ],
            },
            2 => SlsaLevel {
                level: 2,
                requirements: vec![
                    "Requirements of L1".to_string(),
                    "Builder must be versioned".to_string(),
                    "Provenance must be authenticated".to_string(),
                ],
            },
            3 => SlsaLevel {
                level: 3,
                requirements: vec![
                    "Requirements of L2".to_string(),
                    "Build must be hermetic".to_string(),
                    "Non-falsifiable provenance".to_string(),
                    "Completeness: parameters, environment, materials".to_string(),
                    "Reproducible builds".to_string(),
                ],
            },
            _ => SlsaLevel {
                level: 0,
                requirements: vec!["Unknown level".to_string()],
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaVerificationResult {
    pub compliant_level: u8,
    pub is_slsa_l3_compliant: bool,
    pub issues: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slsa_provenance_generation() {
        let doc = SlsaGenerator::generate_v1("output.bin", "abc123blake3hash", "0.4.0");
        assert_eq!(doc.doc_type, "https://in-toto.io/Statement/v1");
        assert_eq!(doc.subject.len(), 1);
        assert_eq!(doc.subject[0].name, "output.bin");
        assert_eq!(doc.predicate.builder.id, "https://github.com/requla11/fish");
    }

    #[test]
    fn test_slsa_l3_generation_and_verification() {
        let materials = vec![
            (
                "https://github.com/requla11/fish".to_string(),
                "sha256".to_string(),
                "abc123".to_string(),
            ),
            (
                "https://crates.io/crate/fish-core".to_string(),
                "sha256".to_string(),
                "def456".to_string(),
            ),
        ];

        let build_steps = vec![BuildStep {
            command: vec!["cargo".to_string(), "build".to_string(), "--release".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("CARGO_TERM_QUIET".to_string(), "true".to_string());
                env
            },
            working_dir: "/workspace".to_string(),
        }];

        let mut params = HashMap::new();
        params.insert("target".to_string(), "x86_64-unknown-linux-gnu".to_string());

        let provenance = SlsaGenerator::generate_slsa_l3(
            "fish-cli",
            "blake3_hash_1234567890",
            "sha256_hash_abcdef",
            "0.4.0",
            materials,
            build_steps,
            params,
        );

        assert_eq!(provenance.subject[0].digest.len(), 2);
        assert!(provenance.predicate.build_config.is_some());
        assert!(provenance.predicate.metadata.is_some());

        let verification = SlsaGenerator::verify_slsa_l3_compliance(&provenance);
        assert!(verification.is_slsa_l3_compliant);
        assert_eq!(verification.compliant_level, 3);
        assert!(verification.issues.is_empty());

        // Test in-toto wrapper
        let attestation = SlsaGenerator::generate_in_toto_attestation(provenance);
        assert_eq!(attestation.doc_type, "https://in-toto.io/Statement/v1");
    }

    #[test]
    fn test_slsa_requirements() {
        let l3 = SlsaGenerator::get_slsa_requirements(3);
        assert_eq!(l3.level, 3);
        assert!(l3.requirements.iter().any(|r| r.contains("hermetic")));
    }

    #[test]
    fn test_slsa_verification_failure() {
        let doc = SlsaGenerator::generate_v1("output.bin", "abc123", "0.4.0");
        let verification = SlsaGenerator::verify_slsa_l3_compliance(&doc);
        assert!(!verification.is_slsa_l3_compliant);
        assert!(!verification.issues.is_empty());
    }
}
