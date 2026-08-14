#![forbid(unsafe_code)]

use crate::error::{CasError, Result};
use serde::{Deserialize, Serialize};

/// Compression algorithms supported by CAS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// Zstandard compression
    Zstd,
    /// Zstandard with maximum compression
    ZstdMax,
    /// Zstandard with fastest compression
    ZstdFast,
}

impl CompressionAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Zstd => "zstd",
            Self::ZstdMax => "zstd-max",
            Self::ZstdFast => "zstd-fast",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(Self::None),
            "zstd" => Some(Self::Zstd),
            "zstd-max" => Some(Self::ZstdMax),
            "zstd-fast" => Some(Self::ZstdFast),
            _ => None,
        }
    }
}

/// Compression level for Zstandard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// Fastest compression, lowest ratio
    Fast = 1,
    /// Default compression
    Default = 3,
    /// Better compression
    High = 10,
    /// Maximum compression, slowest
    Max = 22,
}

impl CompressionLevel {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

/// Compress data using the specified algorithm
pub fn compress(data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>> {
    match algorithm {
        CompressionAlgorithm::None => Ok(data.to_vec()),
        CompressionAlgorithm::Zstd => compress_zstd(data, CompressionLevel::Default),
        CompressionAlgorithm::ZstdMax => compress_zstd(data, CompressionLevel::Max),
        CompressionAlgorithm::ZstdFast => compress_zstd(data, CompressionLevel::Fast),
    }
}

/// Decompress data using the specified algorithm
pub fn decompress(data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>> {
    match algorithm {
        CompressionAlgorithm::None => Ok(data.to_vec()),
        CompressionAlgorithm::Zstd | CompressionAlgorithm::ZstdMax | CompressionAlgorithm::ZstdFast => {
            decompress_zstd(data)
        }
    }
}

fn compress_zstd(data: &[u8], level: CompressionLevel) -> Result<Vec<u8>> {
    let level = level.as_i32();
    zstd::encode_all(data, level)
        .map_err(|e| CasError::Compression(format!("Zstd compression failed: {}", e)))
}

fn decompress_zstd(data: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(data)
        .map_err(|e| CasError::Compression(format!("Zstd decompression failed: {}", e)))
}

/// Estimate compression ratio for a given data
pub fn estimate_compression_ratio(data: &[u8], algorithm: CompressionAlgorithm) -> Option<f64> {
    if algorithm == CompressionAlgorithm::None {
        return None;
    }
    
    // For small data, compression might not help
    if data.len() < 1024 {
        return None;
    }
    
    // Sample a portion to estimate
    let sample_size = std::cmp::min(data.len(), 4096);
    let sample = &data[..sample_size];
    
    match compress(sample, algorithm) {
        Ok(compressed) => {
            let ratio = compressed.len() as f64 / sample.len() as f64;
            Some(ratio)
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compression_none() {
        let data = b"test data";
        let compressed = compress(data, CompressionAlgorithm::None).unwrap();
        assert_eq!(compressed, data);
    }
    
    #[test]
    fn test_zstd_compression() {
        let data = vec![0u8; 10000]; // Highly compressible data
        let compressed = compress(&data, CompressionAlgorithm::Zstd).unwrap();
        assert!(compressed.len() < data.len());
        
        let decompressed = decompress(&compressed, CompressionAlgorithm::Zstd).unwrap();
        assert_eq!(decompressed, data);
    }
    
    #[test]
    fn test_compression_roundtrip() {
        let original = b"This is some test data that should compress reasonably well with zstd compression algorithm because it contains repetitive patterns and text.";
        
        for algorithm in [CompressionAlgorithm::Zstd, CompressionAlgorithm::ZstdMax, CompressionAlgorithm::ZstdFast] {
            let compressed = compress(original, algorithm).unwrap();
            let decompressed = decompress(&compressed, algorithm).unwrap();
            assert_eq!(decompressed, original);
        }
    }
    
    #[test]
    fn test_compression_algorithm_parsing() {
        assert_eq!(CompressionAlgorithm::from_str("zstd"), Some(CompressionAlgorithm::Zstd));
        assert_eq!(CompressionAlgorithm::from_str("none"), Some(CompressionAlgorithm::None));
        assert_eq!(CompressionAlgorithm::from_str("invalid"), None);
    }
    
    #[test]
    fn test_compression_level() {
        assert_eq!(CompressionLevel::Fast.as_i32(), 1);
        assert_eq!(CompressionLevel::Default.as_i32(), 3);
        assert_eq!(CompressionLevel::Max.as_i32(), 22);
    }
}
