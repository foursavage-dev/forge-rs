#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaDigest {
    pub blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaSubject {
    pub name: String,
    pub digest: SlsaDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaInvocation {
    pub config_source: HashMap<String, String>,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaPredicate {
    pub builder: HashMap<String, String>,
    pub build_type: String,
    pub invocation: SlsaInvocation,
    pub materials: Vec<SlsaSubject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaAttestation {
    #[serde(rename = "_type")]
    pub schema_type: String,
    pub predicate_type: String,
    pub subject: Vec<SlsaSubject>,
    pub predicate: SlsaPredicate,
    pub merkle_root: String,
}

pub struct AttestationEngine;

impl AttestationEngine {
    pub fn generate_attestation(
        project_root: &Path,
        output_files: &[PathBuf],
    ) -> io::Result<SlsaAttestation> {
        let mut subjects = Vec::new();
        let mut merkle_hasher = blake3::Hasher::new();

        for file in output_files {
            if file.exists() && file.is_file() {
                let content = fs::read(file)?;
                let hash = blake3::hash(&content).to_hex().to_string();
                merkle_hasher.update(hash.as_bytes());

                let file_name = file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("artifact")
                    .to_string();

                subjects.push(SlsaSubject {
                    name: file_name,
                    digest: SlsaDigest { blake3: hash },
                });
            }
        }

        let mut materials = Vec::new();
        if let Ok(entries) = fs::read_dir(project_root.join("src")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(content) = fs::read(&path) {
                        let hash = blake3::hash(&content).to_hex().to_string();
                        merkle_hasher.update(hash.as_bytes());
                        materials.push(SlsaSubject {
                            name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                            digest: SlsaDigest { blake3: hash },
                        });
                    }
                }
            }
        }

        let merkle_root = merkle_hasher.finalize().to_hex().to_string();

        let mut builder_info = HashMap::new();
        builder_info.insert("id".to_string(), "https://github.com/requla11/forge-rs@v0.1.0".to_string());
        builder_info.insert("timestamp".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs().to_string());

        let mut config_source = HashMap::new();
        config_source.insert("uri".to_string(), project_root.display().to_string());

        let mut parameters = HashMap::new();
        parameters.insert("slsa_level".to_string(), "SLSA_BUILD_LEVEL_3".to_string());

        Ok(SlsaAttestation {
            schema_type: "https://in-toto.io/Statement/v1".to_string(),
            predicate_type: "https://slsa.dev/provenance/v1".to_string(),
            subject: subjects,
            predicate: SlsaPredicate {
                builder: builder_info,
                build_type: "https://forge.build/slsa/v1".to_string(),
                invocation: SlsaInvocation {
                    config_source,
                    parameters,
                },
                materials,
            },
            merkle_root,
        })
    }

    pub fn save_attestation(
        project_root: &Path,
        attestation: &SlsaAttestation,
    ) -> io::Result<PathBuf> {
        let dir = project_root.join(".forge");
        fs::create_dir_all(&dir)?;
        let path = dir.join("attestation.json");
        let content = serde_json::to_string_pretty(attestation)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&path, content)?;
        Ok(path)
    }

    pub fn verify_attestation(
        attestation_path: &Path,
        artifacts_dir: &Path,
    ) -> io::Result<bool> {
        let content = fs::read_to_string(attestation_path)?;
        let attestation: SlsaAttestation = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        for subj in &attestation.subject {
            let artifact_path = artifacts_dir.join(&subj.name);
            if !artifact_path.exists() {
                return Ok(false);
            }
            let bytes = fs::read(&artifact_path)?;
            let current_hash = blake3::hash(&bytes).to_hex().to_string();
            if current_hash != subj.digest.blake3 {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_attestation_generation_and_verification() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), b"pub fn clean() {}").unwrap();

        let out_dir = temp.path().join("target");
        fs::create_dir_all(&out_dir).unwrap();
        let bin_path = out_dir.join("app.exe");
        fs::write(&bin_path, b"pristine binary artifact").unwrap();

        let attestation = AttestationEngine::generate_attestation(temp.path(), std::slice::from_ref(&bin_path)).unwrap();
        assert!(!attestation.merkle_root.is_empty());
        assert_eq!(attestation.subject.len(), 1);

        let att_file = AttestationEngine::save_attestation(temp.path(), &attestation).unwrap();
        assert!(att_file.exists());

        let is_valid = AttestationEngine::verify_attestation(&att_file, &out_dir).unwrap();
        assert!(is_valid);

        fs::write(&bin_path, b"tampered binary artifact").unwrap();
        let is_valid_after_tamper = AttestationEngine::verify_attestation(&att_file, &out_dir).unwrap();
        assert!(!is_valid_after_tamper);
    }
}
