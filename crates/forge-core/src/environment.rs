#![forbid(unsafe_code)]

//! Environment fingerprinting for ABI consistency
//! 
//! This module captures detailed environment information to ensure
//! cache keys include relevant ABI differences across different systems.

use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentFingerprint {
    pub os: String,
    pub os_version: String,
    pub architecture: String,
    pub libc_version: Option<String>,
    pub compiler_versions: HashMap<String, String>,
    pub toolchain_hash: String,
}

impl std::hash::Hash for EnvironmentFingerprint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.os.hash(state);
        self.os_version.hash(state);
        self.architecture.hash(state);
        self.toolchain_hash.hash(state);
        self.libc_version.hash(state);
    }
}

impl EnvironmentFingerprint {
    pub fn capture() -> Self {
        let os = std::env::consts::OS.to_string();
        let os_version = Self::get_os_version();
        let architecture = std::env::consts::ARCH.to_string();
        let libc_version = Self::get_libc_version();
        let compiler_versions = Self::get_compiler_versions();
        let toolchain_hash = Self::compute_toolchain_hash(&compiler_versions, &libc_version);
        
        Self {
            os,
            os_version,
            architecture,
            libc_version,
            compiler_versions,
            toolchain_hash,
        }
    }
    
    fn get_os_version() -> String {
        #[cfg(windows)]
        {
            if let Ok(output) = Command::new("cmd")
                .args(["/c", "ver"])
                .output()
            {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                "Windows Unknown".to_string()
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = Command::new("uname")
                .args(["-r"])
                .output()
            {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                "Linux Unknown".to_string()
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("sw_vers")
                .args(["-productVersion"])
                .output()
            {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                "macOS Unknown".to_string()
            }
        }
        
        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        {
            "Unknown".to_string()
        }
    }
    
    fn get_libc_version() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = Command::new("ldd")
                .args(["--version"])
                .output()
            {
                let version = String::from_utf8_lossy(&output.stdout);
                version.lines().next().map(|line| line.to_string())
            } else {
                None
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }
    
    fn get_compiler_versions() -> HashMap<String, String> {
        let mut versions = HashMap::new();
        
        // GCC
        if let Ok(output) = Command::new("gcc")
            .args(["--version"])
            .output()
        {
            let version = String::from_utf8_lossy(&output.stdout);
            if let Some(first_line) = version.lines().next() {
                versions.insert("gcc".to_string(), first_line.to_string());
            }
        }
        
        // Clang
        if let Ok(output) = Command::new("clang")
            .args(["--version"])
            .output()
        {
            let version = String::from_utf8_lossy(&output.stdout);
            if let Some(first_line) = version.lines().next() {
                versions.insert("clang".to_string(), first_line.to_string());
            }
        }
        
        // Rust
        if let Ok(output) = Command::new("rustc")
            .args(["--version"])
            .output()
        {
            let version = String::from_utf8_lossy(&output.stdout);
            versions.insert("rustc".to_string(), version.trim().to_string());
        }
        
        // Go
        if let Ok(output) = Command::new("go")
            .args(["version"])
            .output()
        {
            let version = String::from_utf8_lossy(&output.stdout);
            versions.insert("go".to_string(), version.trim().to_string());
        }
        
        // Node
        if let Ok(output) = Command::new("node")
            .args(["--version"])
            .output()
        {
            let version = String::from_utf8_lossy(&output.stdout);
            versions.insert("node".to_string(), version.trim().to_string());
        }
        
        versions
    }
    
    fn compute_toolchain_hash(compiler_versions: &HashMap<String, String>, libc_version: &Option<String>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash all compiler versions
        let mut compilers: Vec<_> = compiler_versions.iter().collect();
        compilers.sort_by_key(|(k, _)| *k);
        
        for (compiler, version) in compilers {
            compiler.hash(&mut hasher);
            version.hash(&mut hasher);
        }
        
        // Hash libc version if available
        if let Some(libc) = libc_version {
            libc.hash(&mut hasher);
        }
        
        format!("{:x}", hasher.finish())
    }
    
    pub fn is_compatible_with(&self, other: &EnvironmentFingerprint) -> bool {
        // Strict compatibility check
        self.os == other.os &&
        self.architecture == other.architecture &&
        self.toolchain_hash == other.toolchain_hash
    }
    
    pub fn to_cache_key(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.os,
            self.architecture,
            self.toolchain_hash,
            // Include libc version for Linux for ABI consistency
            self.libc_version.as_deref().unwrap_or("none")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_environment_capture() {
        let fingerprint = EnvironmentFingerprint::capture();
        assert!(!fingerprint.os.is_empty());
        assert!(!fingerprint.architecture.is_empty());
        assert!(!fingerprint.toolchain_hash.is_empty());
    }
    
    #[test]
    fn test_cache_key_generation() {
        let fingerprint = EnvironmentFingerprint::capture();
        let cache_key = fingerprint.to_cache_key();
        assert!(!cache_key.is_empty());
        assert!(cache_key.contains('-'));
    }
    
    #[test]
    fn test_compatibility_check() {
        let fp1 = EnvironmentFingerprint::capture();
        let fp2 = EnvironmentFingerprint::capture();
        
        // Same environment should be compatible
        assert!(fp1.is_compatible_with(&fp2));
    }
}