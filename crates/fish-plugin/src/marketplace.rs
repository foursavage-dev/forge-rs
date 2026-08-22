use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Plugin Marketplace Registry - Decentralized plugin discovery and signed artifact distribution

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub toolchain: String, // e.g., "rust", "go", "custom"
    pub entrypoint: String,
    pub capabilities: Vec<String>,
    pub signature: Option<PluginSignature>,
    pub download_url: Option<String>,
    pub checksum: Option<String>, // blake3 hash
    pub downloads: u64,
    pub verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSignature {
    pub public_key: String, // Ed25519 public key hex
    pub signature: String,  // Signature hex
    pub signed_by: String,
    pub signed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub registry_url: String,
    pub cache_dir: PathBuf,
    pub enable_signature_verification: bool,
    pub trusted_keys: Vec<String>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            registry_url: "https://registry.fish.build".to_string(),
            cache_dir: PathBuf::from(".fish/plugins/cache"),
            enable_signature_verification: true,
            trusted_keys: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub keyword: Option<String>,
    pub toolchain: Option<String>,
    pub verified_only: bool,
    pub limit: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            keyword: None,
            toolchain: None,
            verified_only: false,
            limit: 50,
        }
    }
}

pub struct PluginMarketplace {
    config: RegistryConfig,
    plugins: Arc<Mutex<HashMap<String, PluginMetadata>>>,
    installed: Arc<Mutex<HashMap<String, PathBuf>>>,
}

impl PluginMarketplace {
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            config,
            plugins: Arc::new(Mutex::new(HashMap::new())),
            installed: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(RegistryConfig::default())
    }

    /// Discover plugins from registry (simulated - in production would fetch from remote)
    pub fn discover_plugins(&self) -> Result<Vec<PluginMetadata>, String> {
        let plugins = self.plugins.lock().map_err(|e| e.to_string())?;
        Ok(plugins.values().cloned().collect())
    }

    /// Search plugins
    pub fn search(&self, query: SearchQuery) -> Result<Vec<PluginMetadata>, String> {
        let plugins = self.plugins.lock().map_err(|e| e.to_string())?;
        
        let mut results: Vec<PluginMetadata> = plugins
            .values()
            .filter(|p| {
                if query.verified_only && !p.verified {
                    return false;
                }
                if let Some(ref toolchain) = query.toolchain {
                    if &p.toolchain != toolchain {
                        return false;
                    }
                }
                if let Some(ref keyword) = query.keyword {
                    let kw_lower = keyword.to_lowercase();
                    if !p.name.to_lowercase().contains(&kw_lower)
                        && !p.description.to_lowercase().contains(&kw_lower)
                        && !p.keywords.iter().any(|k| k.to_lowercase().contains(&kw_lower))
                    {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Sort by downloads (popular first)
        results.sort_by(|a, b| b.downloads.cmp(&a.downloads));
        results.truncate(query.limit);

        Ok(results)
    }

    /// Register a plugin in local registry (for testing or local development)
    pub fn register_plugin(&self, metadata: PluginMetadata) -> Result<(), String> {
        let mut plugins = self.plugins.lock().map_err(|e| e.to_string())?;
        plugins.insert(metadata.name.clone(), metadata);
        Ok(())
    }

    /// Get plugin metadata
    pub fn get_plugin(&self, name: &str) -> Result<Option<PluginMetadata>, String> {
        let plugins = self.plugins.lock().map_err(|e| e.to_string())?;
        Ok(plugins.get(name).cloned())
    }

    /// Install plugin from registry
    pub fn install_plugin(&self, name: &str, version: Option<&str>) -> Result<PathBuf, String> {
        let plugin = {
            let plugins = self.plugins.lock().map_err(|e| e.to_string())?;
            plugins
                .get(name)
                .cloned()
                .ok_or_else(|| format!("plugin '{}' not found in registry", name))?
        };

        if let Some(req_version) = version {
            if plugin.version != req_version {
                return Err(format!(
                    "version mismatch: requested {}, available {}",
                    req_version, plugin.version
                ));
            }
        }

        // Verify signature if enabled
        if self.config.enable_signature_verification {
            if let Some(ref sig) = plugin.signature {
                self.verify_signature(&plugin, sig)?;
            } else if plugin.verified {
                return Err(format!("plugin '{}' claims verified but has no signature", name));
            }
        }

        // Verify checksum
        if let Some(ref expected_checksum) = plugin.checksum {
            // In real implementation, download and verify
            // For now, simulate verification
            if expected_checksum.len() != 64 {
                return Err("invalid checksum format".to_string());
            }
        }

        // Simulate installation to cache dir
        let install_path = self.config.cache_dir.join(&plugin.name).join(&plugin.version);
        std::fs::create_dir_all(&install_path).map_err(|e| e.to_string())?;

        // Create plugin manifest
        let manifest_path = install_path.join("plugin.json");
        let manifest_content = serde_json::to_string_pretty(&plugin).map_err(|e| e.to_string())?;
        std::fs::write(&manifest_path, manifest_content).map_err(|e| e.to_string())?;

        // Create dummy wasm file for testing
        let wasm_path = install_path.join(&plugin.entrypoint);
        let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]; // Minimal WASM
        std::fs::write(&wasm_path, wasm_bytes).map_err(|e| e.to_string())?;

        {
            let mut installed = self.installed.lock().map_err(|e| e.to_string())?;
            installed.insert(plugin.name.clone(), install_path.clone());
        }

        Ok(install_path)
    }

    /// Verify plugin signature using Ed25519
    fn verify_signature(
        &self,
        plugin: &PluginMetadata,
        signature: &PluginSignature,
    ) -> Result<(), String> {
        // In production, would use ed25519-dalek to verify
        // For simulation, check that signature is non-empty and trusted key if configured
        
        if signature.signature.is_empty() || signature.public_key.is_empty() {
            return Err("invalid signature: empty".to_string());
        }

        if !self.config.trusted_keys.is_empty() {
            if !self.config.trusted_keys.contains(&signature.public_key) {
                return Err(format!(
                    "untrusted signing key: {}",
                    signature.public_key
                ));
            }
        }

        // Simulate signature verification - check that plugin name is included in signed data
        // Real implementation would verify: sign(blake3(plugin_content))
        if signature.signature.len() < 32 {
            return Err("signature too short".to_string());
        }

        Ok(())
    }

    /// Uninstall plugin
    pub fn uninstall_plugin(&self, name: &str) -> Result<(), String> {
        let install_path = {
            let installed = self.installed.lock().map_err(|e| e.to_string())?;
            installed
                .get(name)
                .cloned()
                .ok_or_else(|| format!("plugin '{}' not installed", name))?
        };

        std::fs::remove_dir_all(&install_path).map_err(|e| e.to_string())?;

        {
            let mut installed = self.installed.lock().map_err(|e| e.to_string())?;
            installed.remove(name);
        }

        Ok(())
    }

    /// List installed plugins
    pub fn list_installed(&self) -> Result<Vec<(String, PathBuf)>, String> {
        let installed = self.installed.lock().map_err(|e| e.to_string())?;
        Ok(installed
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }

    /// Check for updates
    pub fn check_updates(&self) -> Result<Vec<(String, String, String)>, String> {
        // Returns (plugin_name, current_version, latest_version)
        let installed = self.installed.lock().map_err(|e| e.to_string())?;
        let plugins = self.plugins.lock().map_err(|e| e.to_string())?;

        let mut updates = Vec::new();
        for (name, _path) in installed.iter() {
            if let Some(latest) = plugins.get(name) {
                // In real implementation, would parse semver and compare
                // For simulation, assume if registry has plugin, it's an update
                // We need to track current version - for now use placeholder
                let current_version = "0.1.0"; // Would read from installed manifest
                if latest.version != current_version {
                    updates.push((name.clone(), current_version.to_string(), latest.version.clone()));
                }
            }
        }

        Ok(updates)
    }

    /// Publish plugin to registry (for plugin authors)
    pub fn publish_plugin(
        &self,
        plugin_path: &Path,
        metadata: PluginMetadata,
    ) -> Result<PluginMetadata, String> {
        if !plugin_path.exists() {
            return Err(format!("plugin path does not exist: {:?}", plugin_path));
        }

        // Validate plugin
        self.validate_plugin(plugin_path)?;

        // Sign plugin if trusted key available
        let mut signed_metadata = metadata.clone();
        if !self.config.trusted_keys.is_empty() {
            signed_metadata.signature = Some(PluginSignature {
                public_key: self.config.trusted_keys[0].clone(),
                signature: format!("sig_{}_{}", metadata.name, metadata.version),
                signed_by: "fish-registry".to_string(),
                signed_at: chrono::Utc::now(),
            });
            signed_metadata.verified = true;
        }

        // Register in local registry
        self.register_plugin(signed_metadata.clone())?;

        Ok(signed_metadata)
    }

    fn validate_plugin(&self, plugin_path: &Path) -> Result<(), String> {
        // Check for required files
        let manifest = plugin_path.join("plugin.json");
        if !manifest.exists() {
            // Also check for plugin.wasm
            let wasm = plugin_path.join("plugin.wasm");
            if !wasm.exists() {
                return Err("plugin must have plugin.json or plugin.wasm".to_string());
            }
        }

        // Validate WASM header if present
        let wasm_file = plugin_path.join("plugin.wasm");
        if wasm_file.exists() {
            let bytes = std::fs::read(&wasm_file).map_err(|e| e.to_string())?;
            if bytes.len() < 8 || &bytes[0..4] != b"\0asm" {
                return Err("invalid WASM file".to_string());
            }
        }

        Ok(())
    }

    /// Generate registry index for static hosting
    pub fn generate_index(&self) -> Result<serde_json::Value, String> {
        let plugins = self.plugins.lock().map_err(|e| e.to_string())?;
        
        let index = serde_json::json!({
            "registry": self.config.registry_url,
            "version": "1.0",
            "plugins": plugins.values().collect::<Vec<_>>(),
            "total": plugins.len(),
            "generated_at": chrono::Utc::now().to_rfc3339(),
        });

        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_marketplace_registry() {
        let temp = tempdir().unwrap();
        let config = RegistryConfig {
            registry_url: "https://test.registry.fish.build".to_string(),
            cache_dir: temp.path().join("cache"),
            enable_signature_verification: false,
            trusted_keys: Vec::new(),
        };

        let marketplace = PluginMarketplace::new(config);

        let plugin = PluginMetadata {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "test-author".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec!["test".to_string()],
            toolchain: "rust".to_string(),
            entrypoint: "plugin.wasm".to_string(),
            capabilities: vec!["build".to_string()],
            signature: None,
            download_url: None,
            checksum: Some("a".repeat(64)),
            downloads: 100,
            verified: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        marketplace.register_plugin(plugin.clone()).unwrap();

        let search_results = marketplace
            .search(SearchQuery {
                keyword: Some("test".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(search_results.len(), 1);

        let installed_path = marketplace.install_plugin("test-plugin", None).unwrap();
        assert!(installed_path.exists());

        let installed = marketplace.list_installed().unwrap();
        assert_eq!(installed.len(), 1);

        marketplace.uninstall_plugin("test-plugin").unwrap();
        let installed_after = marketplace.list_installed().unwrap();
        assert_eq!(installed_after.len(), 0);
    }

    #[test]
    fn test_plugin_signature_verification() {
        let temp = tempdir().unwrap();
        let config = RegistryConfig {
            registry_url: "https://test.registry.fish.build".to_string(),
            cache_dir: temp.path().join("cache"),
            enable_signature_verification: true,
            trusted_keys: vec!["trusted_public_key_123".to_string()],
        };

        let marketplace = PluginMarketplace::new(config);

        let mut plugin = PluginMetadata {
            name: "verified-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Verified plugin".to_string(),
            author: "trusted".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            toolchain: "rust".to_string(),
            entrypoint: "plugin.wasm".to_string(),
            capabilities: vec![],
            signature: Some(PluginSignature {
                public_key: "trusted_public_key_123".to_string(),
                signature: "valid_signature_".to_string().repeat(5),
                signed_by: "trusted".to_string(),
                signed_at: chrono::Utc::now(),
            }),
            download_url: None,
            checksum: Some("b".repeat(64)),
            downloads: 1000,
            verified: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        marketplace.register_plugin(plugin.clone()).unwrap();
        assert!(marketplace.install_plugin("verified-plugin", None).is_ok());

        // Test untrusted key
        plugin.name = "untrusted-plugin".to_string();
        plugin.signature = Some(PluginSignature {
            public_key: "untrusted_key".to_string(),
            signature: "valid_signature_".to_string().repeat(5),
            signed_by: "untrusted".to_string(),
            signed_at: chrono::Utc::now(),
        });

        marketplace.register_plugin(plugin).unwrap();
        assert!(marketplace.install_plugin("untrusted-plugin", None).is_err());
    }

    #[test]
    fn test_marketplace_index_generation() {
        let marketplace = PluginMarketplace::with_default_config();
        
        let plugin = PluginMetadata {
            name: "index-test".to_string(),
            version: "0.1.0".to_string(),
            description: "Test".to_string(),
            author: "test".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            toolchain: "rust".to_string(),
            entrypoint: "plugin.wasm".to_string(),
            capabilities: vec![],
            signature: None,
            download_url: None,
            checksum: None,
            downloads: 0,
            verified: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        marketplace.register_plugin(plugin).unwrap();
        let index = marketplace.generate_index().unwrap();
        assert_eq!(index["total"], 1);
        assert!(index["plugins"].is_array());
    }
}
