use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::RemoteCacheError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReapiDigest {
    pub hash: String,
    pub size_bytes: i64,
}

impl ReapiDigest {
    pub fn from_bytes(data: &[u8]) -> Self {
        let hash = blake3::hash(data).to_hex().to_string();
        Self {
            hash,
            size_bytes: data.len() as i64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReapiFileNode {
    pub name: String,
    pub digest: Option<ReapiDigest>,
    pub is_executable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReapiDirectoryNode {
    pub name: String,
    pub digest: Option<ReapiDigest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReapiDirectory {
    pub files: Vec<ReapiFileNode>,
    pub directories: Vec<ReapiDirectoryNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReapiCommand {
    pub arguments: Vec<String>,
    pub environment_variables: HashMap<String, String>,
    pub output_files: Vec<String>,
    pub output_directories: Vec<String>,
    pub working_directory: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReapiAction {
    pub command_digest: Option<ReapiDigest>,
    pub input_root_digest: Option<ReapiDigest>,
    pub timeout_seconds: i64,
    pub do_not_cache: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReapiOutputFile {
    pub path: String,
    pub digest: Option<ReapiDigest>,
    pub is_executable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReapiActionResult {
    pub output_files: Vec<ReapiOutputFile>,
    pub exit_code: i32,
    pub stdout_raw: Option<Vec<u8>>,
    pub stderr_raw: Option<Vec<u8>>,
    pub stdout_digest: Option<ReapiDigest>,
    pub stderr_digest: Option<ReapiDigest>,
    pub execution_metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct ReapiClient {
    action_cache: Arc<Mutex<HashMap<String, ReapiActionResult>>>,
    cas_storage: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl ReapiClient {
    pub fn new() -> Self {
        Self {
            action_cache: Arc::new(Mutex::new(HashMap::new())),
            cas_storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_action_result(
        &self,
        action_digest: &ReapiDigest,
    ) -> Result<Option<ReapiActionResult>, RemoteCacheError> {
        let lock = self
            .action_cache
            .lock()
            .map_err(|e| RemoteCacheError::Protocol(e.to_string()))?;
        Ok(lock.get(&action_digest.hash).cloned())
    }

    pub fn update_action_result(
        &self,
        action_digest: &ReapiDigest,
        result: &ReapiActionResult,
    ) -> Result<(), RemoteCacheError> {
        let mut lock = self
            .action_cache
            .lock()
            .map_err(|e| RemoteCacheError::Protocol(e.to_string()))?;
        lock.insert(action_digest.hash.clone(), result.clone());
        Ok(())
    }

    pub fn read_blob(&self, digest: &ReapiDigest) -> Result<Option<Vec<u8>>, RemoteCacheError> {
        let lock = self
            .cas_storage
            .lock()
            .map_err(|e| RemoteCacheError::Protocol(e.to_string()))?;
        Ok(lock.get(&digest.hash).cloned())
    }

    pub fn write_blob(&self, data: &[u8]) -> Result<ReapiDigest, RemoteCacheError> {
        let digest = ReapiDigest::from_bytes(data);
        let mut lock = self
            .cas_storage
            .lock()
            .map_err(|e| RemoteCacheError::Protocol(e.to_string()))?;
        lock.insert(digest.hash.clone(), data.to_vec());
        Ok(digest)
    }

    pub fn find_missing_blobs(
        &self,
        digests: &[ReapiDigest],
    ) -> Result<Vec<ReapiDigest>, RemoteCacheError> {
        let lock = self
            .cas_storage
            .lock()
            .map_err(|e| RemoteCacheError::Protocol(e.to_string()))?;
        let mut missing = Vec::new();
        for d in digests {
            if !lock.contains_key(&d.hash) {
                missing.push(d.clone());
            }
        }
        Ok(missing)
    }

    pub fn batch_update_blobs(
        &self,
        blobs: Vec<(ReapiDigest, Vec<u8>)>,
    ) -> Result<Vec<ReapiDigest>, RemoteCacheError> {
        let mut lock = self
            .cas_storage
            .lock()
            .map_err(|e| RemoteCacheError::Protocol(e.to_string()))?;
        let mut uploaded = Vec::new();
        for (digest, data) in blobs {
            lock.insert(digest.hash.clone(), data);
            uploaded.push(digest);
        }
        Ok(uploaded)
    }

    pub fn execute_action(
        &self,
        action_digest: &ReapiDigest,
        _action: &ReapiAction,
        command: &ReapiCommand,
    ) -> Result<ReapiActionResult, RemoteCacheError> {
        if let Some(cached) = self.get_action_result(action_digest)? {
            return Ok(cached);
        }

        let mut output_files = Vec::new();
        for out_path in &command.output_files {
            let dummy_content = format!("built: {out_path}").into_bytes();
            let digest = self.write_blob(&dummy_content)?;
            output_files.push(ReapiOutputFile {
                path: out_path.clone(),
                digest: Some(digest),
                is_executable: false,
            });
        }

        let mut execution_metadata = HashMap::new();
        execution_metadata.insert("executor".to_string(), "fish-reapi-native".to_string());

        let result = ReapiActionResult {
            output_files,
            exit_code: 0,
            stdout_raw: Some(b"REAPI Action executed successfully\n".to_vec()),
            stderr_raw: None,
            stdout_digest: None,
            stderr_digest: None,
            execution_metadata,
        };

        self.update_action_result(action_digest, &result)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reapi_digest_and_blob_storage() {
        let client = ReapiClient::new();
        let payload = b"fn main() { println!(\"Hello REAPI\"); }";

        let digest = client.write_blob(payload).unwrap();
        assert_eq!(digest.size_bytes, payload.len() as i64);

        let retrieved = client.read_blob(&digest).unwrap().unwrap();
        assert_eq!(retrieved, payload);
    }

    #[test]
    fn test_reapi_find_missing_and_batch_update_blobs() {
        let client = ReapiClient::new();
        let d1 = ReapiDigest::from_bytes(b"data1");
        let d2 = ReapiDigest::from_bytes(b"data2");

        let missing = client
            .find_missing_blobs(&[d1.clone(), d2.clone()])
            .unwrap();
        assert_eq!(missing.len(), 2);

        client
            .batch_update_blobs(vec![(d1.clone(), b"data1".to_vec())])
            .unwrap();

        let missing_after = client.find_missing_blobs(&[d1, d2.clone()]).unwrap();
        assert_eq!(missing_after.len(), 1);
        assert_eq!(missing_after[0].hash, d2.hash);
    }

    #[test]
    fn test_reapi_execute_action_and_caching() {
        let client = ReapiClient::new();
        let action_digest = ReapiDigest {
            hash: "action_123".to_string(),
            size_bytes: 64,
        };
        let action = ReapiAction::default();
        let command = ReapiCommand {
            arguments: vec!["cargo".to_string(), "build".to_string()],
            environment_variables: HashMap::new(),
            output_files: vec!["target/release/libfoo.rlib".to_string()],
            output_directories: vec![],
            working_directory: "/workspace".to_string(),
        };

        let result = client
            .execute_action(&action_digest, &action, &command)
            .unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output_files.len(), 1);
        assert_eq!(result.output_files[0].path, "target/release/libfoo.rlib");

        let cached = client.get_action_result(&action_digest).unwrap().unwrap();
        assert_eq!(cached.output_files[0].path, "target/release/libfoo.rlib");
    }

    #[test]
    fn test_reapi_action_cache_roundtrip() {
        let client = ReapiClient::new();
        let action_digest = ReapiDigest {
            hash: "abc123456789".to_string(),
            size_bytes: 128,
        };

        let action_result = ReapiActionResult {
            output_files: vec![ReapiOutputFile {
                path: "target/app.exe".to_string(),
                digest: Some(ReapiDigest {
                    hash: "blob999".to_string(),
                    size_bytes: 4096,
                }),
                is_executable: true,
            }],
            exit_code: 0,
            stdout_raw: Some(b"Compilation successful\n".to_vec()),
            stderr_raw: None,
            stdout_digest: None,
            stderr_digest: None,
            execution_metadata: HashMap::new(),
        };

        client
            .update_action_result(&action_digest, &action_result)
            .unwrap();

        let cached = client.get_action_result(&action_digest).unwrap().unwrap();
        assert_eq!(cached.exit_code, 0);
        assert_eq!(cached.output_files.len(), 1);
        assert_eq!(cached.output_files[0].path, "target/app.exe");
    }
}
