use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub hash: String,
    pub offset: usize,
    pub length: usize,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkManifest {
    pub total_size: usize,
    pub chunk_hashes: Vec<String>,
    pub chunk_lengths: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct FastCdcChunker {
    pub min_size: usize,
    pub avg_size: usize,
    pub max_size: usize,
}

impl Default for FastCdcChunker {
    fn default() -> Self {
        Self {
            min_size: 16 * 1024,
            avg_size: 64 * 1024,
            max_size: 256 * 1024,
        }
    }
}

impl FastCdcChunker {
    pub fn new(min_size: usize, avg_size: usize, max_size: usize) -> Self {
        Self {
            min_size,
            avg_size,
            max_size,
        }
    }

    pub fn chunk_data(&self, data: &[u8]) -> (Vec<Chunk>, ChunkManifest) {
        let mut chunks = Vec::new();
        let mut chunk_hashes = Vec::new();
        let mut chunk_lengths = Vec::new();
        let mut offset = 0;
        let len = data.len();

        if len == 0 {
            let empty_hash = blake3::hash(&[]).to_hex().to_string();
            let chunk = Chunk {
                hash: empty_hash.clone(),
                offset: 0,
                length: 0,
                data: Vec::new(),
            };
            let manifest = ChunkManifest {
                total_size: 0,
                chunk_hashes: vec![empty_hash],
                chunk_lengths: vec![0],
            };
            return (vec![chunk], manifest);
        }

        while offset < len {
            let remaining = len - offset;
            let mut chunk_len = if remaining <= self.min_size {
                remaining
            } else {
                let limit = remaining.min(self.max_size);
                let mut cut = self.min_size;
                let mask = (self.avg_size - 1) as u32;

                while cut < limit {
                    let byte = data[offset + cut];
                    let hash_val = (byte as u32).wrapping_mul(0x5bd1e995);
                    if (hash_val & mask) == 0 {
                        break;
                    }
                    cut += 1;
                }
                cut
            };

            if chunk_len == 0 {
                chunk_len = remaining;
            }

            let slice = &data[offset..offset + chunk_len];
            let hash = blake3::hash(slice).to_hex().to_string();

            chunks.push(Chunk {
                hash: hash.clone(),
                offset,
                length: chunk_len,
                data: slice.to_vec(),
            });

            chunk_hashes.push(hash);
            chunk_lengths.push(chunk_len);
            offset += chunk_len;
        }

        let manifest = ChunkManifest {
            total_size: len,
            chunk_hashes,
            chunk_lengths,
        };

        (chunks, manifest)
    }

    pub fn reconstruct_from_chunks(
        manifest: &ChunkManifest,
        chunk_map: &BTreeMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::with_capacity(manifest.total_size);
        for hash in &manifest.chunk_hashes {
            if let Some(chunk_bytes) = chunk_map.get(hash) {
                buffer.extend_from_slice(chunk_bytes);
            } else {
                return Err(format!("Missing chunk with hash: {hash}"));
            }
        }
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_and_reconstruct_roundtrip() {
        let chunker = FastCdcChunker::new(64, 128, 256);
        let sample_data: Vec<u8> = (0..1024).map(|i| (i % 251) as u8).collect();

        let (chunks, manifest) = chunker.chunk_data(&sample_data);
        assert!(!chunks.is_empty());
        assert_eq!(manifest.total_size, 1024);

        let mut map = BTreeMap::new();
        for c in chunks {
            map.insert(c.hash.clone(), c.data);
        }

        let restored = FastCdcChunker::reconstruct_from_chunks(&manifest, &map).unwrap();
        assert_eq!(restored, sample_data);
    }

    #[test]
    fn test_empty_data_chunking() {
        let chunker = FastCdcChunker::default();
        let (chunks, manifest) = chunker.chunk_data(&[]);
        assert_eq!(chunks.len(), 1);
        assert_eq!(manifest.total_size, 0);
    }
}
