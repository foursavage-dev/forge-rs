use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactChunk {
    pub index: u32,
    pub offset: u64,
    pub size: usize,
    pub checksum: String,
}

#[derive(Debug, Clone)]
pub struct ChunkManifest {
    pub artifact_hash: String,
    pub total_bytes: u64,
    pub chunk_size: usize,
    pub chunks: Vec<ArtifactChunk>,
}

impl ChunkManifest {
    pub fn create_from_bytes(artifact_hash: &str, data: &[u8], chunk_size: usize) -> Self {
        let chunk_size = if chunk_size == 0 {
            1024 * 1024
        } else {
            chunk_size
        };
        let mut chunks = Vec::new();
        let total_bytes = data.len() as u64;

        let mut offset = 0;
        let mut idx = 0;
        while offset < total_bytes {
            let end = (offset + chunk_size as u64).min(total_bytes) as usize;
            let slice = &data[offset as usize..end];
            let hash = blake3::hash(slice).to_hex().to_string();

            chunks.push(ArtifactChunk {
                index: idx,
                offset,
                size: slice.len(),
                checksum: hash,
            });

            offset += slice.len() as u64;
            idx += 1;
        }

        Self {
            artifact_hash: artifact_hash.to_string(),
            total_bytes,
            chunk_size,
            chunks,
        }
    }

    pub fn verify_chunk(&self, chunk_index: usize, chunk_data: &[u8]) -> bool {
        if let Some(chunk) = self.chunks.get(chunk_index) {
            if chunk.size != chunk_data.len() {
                return false;
            }
            let hash = blake3::hash(chunk_data).to_hex().to_string();
            chunk.checksum == hash
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChunkBitfield {
    words: Vec<u64>,
    total_chunks: usize,
}

impl ChunkBitfield {
    pub fn new(total_chunks: usize) -> Self {
        let num_words = total_chunks.div_ceil(64);
        Self {
            words: vec![0u64; num_words],
            total_chunks,
        }
    }

    pub fn set(&mut self, index: usize, val: bool) {
        if index < self.total_chunks {
            let word_idx = index / 64;
            let bit_idx = index % 64;
            if val {
                self.words[word_idx] |= 1u64 << bit_idx;
            } else {
                self.words[word_idx] &= !(1u64 << bit_idx);
            }
        }
    }

    pub fn get(&self, index: usize) -> bool {
        if index >= self.total_chunks {
            return false;
        }
        let word_idx = index / 64;
        let bit_idx = index % 64;
        (self.words[word_idx] & (1u64 << bit_idx)) != 0
    }

    pub fn is_complete(&self) -> bool {
        if self.total_chunks == 0 {
            return false;
        }
        let full_words = self.total_chunks / 64;
        for &w in &self.words[..full_words] {
            if w != u64::MAX {
                return false;
            }
        }
        let remainder = self.total_chunks % 64;
        if remainder > 0 {
            let mask = (1u64 << remainder) - 1;
            (self.words[full_words] & mask) == mask
        } else {
            true
        }
    }

    pub fn count_available(&self) -> usize {
        let mut count = 0;
        let full_words = self.total_chunks / 64;
        for &w in &self.words[..full_words] {
            count += w.count_ones() as usize;
        }
        let remainder = self.total_chunks % 64;
        if remainder > 0 && full_words < self.words.len() {
            let mask = (1u64 << remainder) - 1;
            count += (self.words[full_words] & mask).count_ones() as usize;
        }
        count
    }

    pub fn missing_indices(&self) -> Vec<usize> {
        let mut missing =
            Vec::with_capacity(self.total_chunks.saturating_sub(self.count_available()));
        for i in 0..self.total_chunks {
            if !self.get(i) {
                missing.push(i);
            }
        }
        missing
    }
}

#[derive(Debug, Clone)]
pub struct LanPeerNode {
    pub peer_id: String,
    pub address: SocketAddr,
    pub available_artifacts: HashSet<String>,
    pub chunk_bitfields: HashMap<String, ChunkBitfield>,
}

#[derive(Debug, Clone, Default)]
pub struct LanPeerRegistry {
    peers: Arc<RwLock<HashMap<String, LanPeerNode>>>,
}

impl LanPeerRegistry {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_peer(&self, peer_id: &str, address: SocketAddr) {
        let mut map = self.peers.write().unwrap();
        map.insert(
            peer_id.to_string(),
            LanPeerNode {
                peer_id: peer_id.to_string(),
                address,
                available_artifacts: HashSet::new(),
                chunk_bitfields: HashMap::new(),
            },
        );
    }

    pub fn announce_artifact(&self, peer_id: &str, artifact_hash: &str) {
        let mut map = self.peers.write().unwrap();
        if let Some(peer) = map.get_mut(peer_id) {
            peer.available_artifacts.insert(artifact_hash.to_string());
        }
    }

    pub fn update_peer_bitfield(
        &self,
        peer_id: &str,
        artifact_hash: &str,
        bitfield: ChunkBitfield,
    ) {
        let mut map = self.peers.write().unwrap();
        if let Some(peer) = map.get_mut(peer_id) {
            if bitfield.is_complete() {
                peer.available_artifacts.insert(artifact_hash.to_string());
            }
            peer.chunk_bitfields
                .insert(artifact_hash.to_string(), bitfield);
        }
    }

    pub fn locate_artifact_peers(&self, artifact_hash: &str) -> Vec<SocketAddr> {
        let map = self.peers.read().unwrap();
        let mut found = Vec::new();
        for peer in map.values() {
            if peer.available_artifacts.contains(artifact_hash) {
                found.push(peer.address);
            }
        }
        found
    }

    pub fn locate_chunk_providers(
        &self,
        artifact_hash: &str,
        chunk_index: usize,
    ) -> Vec<SocketAddr> {
        let map = self.peers.read().unwrap();
        let mut found = Vec::new();
        for peer in map.values() {
            if peer.available_artifacts.contains(artifact_hash) {
                found.push(peer.address);
            } else if let Some(bf) = peer.chunk_bitfields.get(artifact_hash)
                && bf.get(chunk_index)
            {
                found.push(peer.address);
            }
        }
        found
    }

    pub fn peer_count(&self) -> usize {
        self.peers.read().unwrap().len()
    }
}

pub struct P2PArtifactReassembler {
    manifest: ChunkManifest,
    buffer: Vec<u8>,
    bitfield: ChunkBitfield,
}

impl P2PArtifactReassembler {
    pub fn new(manifest: ChunkManifest) -> Self {
        let total = manifest.total_bytes as usize;
        let num_chunks = manifest.chunks.len();
        Self {
            manifest,
            buffer: vec![0u8; total],
            bitfield: ChunkBitfield::new(num_chunks),
        }
    }

    pub fn receive_chunk(&mut self, chunk_index: usize, chunk_data: &[u8]) -> Result<bool, String> {
        if !self.manifest.verify_chunk(chunk_index, chunk_data) {
            return Err(format!("Chunk {chunk_index} checksum mismatch or corrupt"));
        }

        let chunk = &self.manifest.chunks[chunk_index];
        let start = chunk.offset as usize;
        let end = start + chunk.size;
        self.buffer[start..end].copy_from_slice(chunk_data);
        self.bitfield.set(chunk_index, true);

        Ok(self.bitfield.is_complete())
    }

    pub fn missing_chunks(&self) -> Vec<usize> {
        self.bitfield.missing_indices()
    }

    pub fn is_ready(&self) -> bool {
        self.bitfield.is_complete()
    }

    pub fn finish(self) -> Result<Vec<u8>, String> {
        if !self.is_ready() {
            return Err("Artifact reassembly incomplete: missing chunks".to_string());
        }
        let whole_hash = blake3::hash(&self.buffer).to_hex().to_string();
        if whole_hash != self.manifest.artifact_hash {
            return Err("Reassembled artifact full hash mismatch".to_string());
        }
        Ok(self.buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lan_peer_discovery_and_artifact_lookup() {
        let registry = LanPeerRegistry::new();
        let addr1: SocketAddr = "192.168.1.50:9527".parse().unwrap();
        let addr2: SocketAddr = "192.168.1.51:9527".parse().unwrap();

        registry.register_peer("peer_alpha", addr1);
        registry.register_peer("peer_beta", addr2);

        registry.announce_artifact("peer_alpha", "hash_blake3_1234");
        assert_eq!(
            registry.locate_artifact_peers("hash_blake3_1234"),
            vec![addr1]
        );
        assert_eq!(registry.locate_artifact_peers("missing_hash").len(), 0);
    }

    #[test]
    fn test_chunk_manifest_and_reassembly() {
        let original_data = b"Hello, this is a large binary artifact compiled by Fish engine! Multiple chunks test.";
        let whole_hash = blake3::hash(original_data).to_hex().to_string();

        let manifest = ChunkManifest::create_from_bytes(&whole_hash, original_data, 16);
        assert_eq!(manifest.chunks.len(), original_data.len().div_ceil(16));

        let mut reassembler = P2PArtifactReassembler::new(manifest.clone());
        assert!(!reassembler.is_ready());
        assert_eq!(reassembler.missing_chunks().len(), manifest.chunks.len());

        for (i, chunk) in manifest.chunks.iter().enumerate() {
            let start = chunk.offset as usize;
            let end = start + chunk.size;
            let chunk_data = &original_data[start..end];
            let is_complete = reassembler.receive_chunk(i, chunk_data).unwrap();
            if i == manifest.chunks.len() - 1 {
                assert!(is_complete);
            }
        }

        assert!(reassembler.is_ready());
        let reassembled = reassembler.finish().unwrap();
        assert_eq!(reassembled, original_data);
    }

    #[test]
    fn test_corrupt_chunk_detection() {
        let original_data = b"Integrity check payload for P2P chunk testing";
        let whole_hash = blake3::hash(original_data).to_hex().to_string();
        let manifest = ChunkManifest::create_from_bytes(&whole_hash, original_data, 10);

        let mut reassembler = P2PArtifactReassembler::new(manifest);
        let corrupted = b"corrupted!";
        let err = reassembler.receive_chunk(0, corrupted);
        assert!(err.is_err());
    }

    #[test]
    fn test_partial_bitfield_peer_location() {
        let registry = LanPeerRegistry::new();
        let addr1: SocketAddr = "10.0.0.1:9000".parse().unwrap();
        let addr2: SocketAddr = "10.0.0.2:9000".parse().unwrap();

        registry.register_peer("node_1", addr1);
        registry.register_peer("node_2", addr2);

        let mut bf1 = ChunkBitfield::new(4);
        bf1.set(0, true);
        bf1.set(1, true);

        let mut bf2 = ChunkBitfield::new(4);
        bf2.set(2, true);
        bf2.set(3, true);

        registry.update_peer_bitfield("node_1", "art_xyz", bf1);
        registry.update_peer_bitfield("node_2", "art_xyz", bf2);

        let p0 = registry.locate_chunk_providers("art_xyz", 0);
        assert_eq!(p0, vec![addr1]);

        let p3 = registry.locate_chunk_providers("art_xyz", 3);
        assert_eq!(p3, vec![addr2]);
    }
}
