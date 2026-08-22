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
                // `avg_size.max(1)` avoids an underflow/divide-by-zero when a
                // caller constructs the chunker with `avg_size == 0`. A modulo
                // (instead of `hash & (avg_size - 1)`) also stays correct for
                // non-power-of-two averages, where the old bitmask was not
                // equivalent to `hash % avg_size`.
                let divisor = self.avg_size.max(1) as u64;

                while cut < limit {
                    // Hash a small window (up to 8 bytes) rather than a single
                    // byte: a one-byte hash has only 256 distinct values, so
                    // for `avg_size > 256` boundaries essentially never fire
                    // and every chunk collapses to `max_size`. A 64-bit hash
                    // spreads boundaries at the intended `1 / avg_size` rate.
                    let mut hash_val: u64 = 0;
                    let window = &data[offset + cut..(offset + cut + 8).min(len)];
                    for &byte in window {
                        hash_val = hash_val.wrapping_mul(0x5bd1e995).wrapping_add(byte as u64);
                    }
                    if hash_val.is_multiple_of(divisor) {
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

    #[test]
    fn zero_avg_size_does_not_panic_and_still_roundtrips() {
        let chunker = FastCdcChunker::new(64, 0, 256);
        let sample: Vec<u8> = (0..1024).map(|i| (i % 253) as u8).collect();

        let (chunks, manifest) = chunker.chunk_data(&sample);
        assert!(!chunks.is_empty());

        let mut map = BTreeMap::new();
        for c in chunks {
            map.insert(c.hash, c.data);
        }
        let restored = FastCdcChunker::reconstruct_from_chunks(&manifest, &map).unwrap();
        assert_eq!(restored, sample);
    }

    #[test]
    fn non_power_of_two_avg_size_roundtrips() {
        let chunker = FastCdcChunker::new(16, 100, 512);
        let sample: Vec<u8> = (0..4096).map(|i| ((i * 37 + 11) % 256) as u8).collect();

        let (chunks, manifest) = chunker.chunk_data(&sample);
        assert!(!chunks.is_empty());
        assert_eq!(manifest.total_size, sample.len());

        let mut map = BTreeMap::new();
        for c in chunks {
            map.insert(c.hash, c.data);
        }
        let restored = FastCdcChunker::reconstruct_from_chunks(&manifest, &map).unwrap();
        assert_eq!(restored, sample);
    }

    #[test]
    fn large_average_size_produces_avg_sized_chunks_not_max_sized() {
        // With avg_size = 1024 (> 256) and high-entropy input, a one-byte
        // hash would collapse to chunks of ~min_size + 256 bytes; the
        // multi-byte window hash must spread boundaries at ~1/1024, so the
        // average chunk length tracks `avg_size` instead of `max_size`. The
        // input is deterministic (xorshift64), so this cannot flake.
        let chunker = FastCdcChunker::new(16, 1024, 4096);

        let mut state: u64 = 0x1234_5678_9abc_def0;
        let sample: Vec<u8> = (0..524288)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state >> 56) as u8
            })
            .collect();

        let (chunks, manifest) = chunker.chunk_data(&sample);
        assert_eq!(manifest.total_size, sample.len());
        let count = chunks.len();

        // ~524288 / (16 + 1024) ≈ 493 chunks expected; the old single-byte
        // hash would yield ~1900. A generous window separates the two while
        // tolerating hash-distribution variance.
        assert!(
            (200..=1500).contains(&count),
            "expected avg-sized chunks, got {count}"
        );
    }
}
