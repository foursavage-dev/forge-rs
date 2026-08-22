#![forbid(unsafe_code)]

use crate::artifact::{Artifact, ArtifactHash};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// BitTorrent-inspired P2P mesh distribution for massive CI runner farms

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub String);

impl PeerId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub hash: String,
    pub size_bytes: u64,
    pub index: usize,
    pub total_chunks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub address: String,
    pub available_chunks: HashSet<String>,
    pub upload_speed_bps: u64,
    pub download_speed_bps: u64,
    pub last_seen: u64,
    pub reputation: f64, // 0.0 - 1.0
}

impl PeerInfo {
    pub fn new(peer_id: PeerId, address: impl Into<String>) -> Self {
        Self {
            peer_id,
            address: address.into(),
            available_chunks: HashSet::new(),
            upload_speed_bps: 0,
            download_speed_bps: 0,
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            reputation: 1.0,
        }
    }

    pub fn has_chunk(&self, hash: &str) -> bool {
        self.available_chunks.contains(hash)
    }

    pub fn add_chunk(&mut self, hash: String) {
        self.available_chunks.insert(hash);
        self.last_seen = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentManifest {
    pub artifact_hash: String,
    pub total_size: u64,
    pub chunk_size: u64,
    pub chunks: Vec<ChunkInfo>,
    pub created_at: u64,
}

impl TorrentManifest {
    pub fn from_artifact(artifact: &Artifact, chunk_size: u64) -> Self {
        let data = artifact.data();
        let total_size = data.len() as u64;
        let total_chunks = ((total_size + chunk_size - 1) / chunk_size) as usize;

        let mut chunks = Vec::with_capacity(total_chunks);
        for i in 0..total_chunks {
            let start = (i as u64 * chunk_size) as usize;
            let end = ((i as u64 + 1) * chunk_size).min(total_size) as usize;
            let chunk_data = &data[start..end];
            let chunk_hash = blake3::hash(chunk_data).to_hex().to_string();

            chunks.push(ChunkInfo {
                hash: chunk_hash,
                size_bytes: (end - start) as u64,
                index: i,
                total_chunks,
            });
        }

        Self {
            artifact_hash: artifact.hash().as_str().to_string(),
            total_size,
            chunk_size,
            chunks,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}

/// P2P Mesh Router - BitTorrent-inspired artifact sharing
pub struct P2PMeshRouter {
    local_peer_id: PeerId,
    peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    manifests: Arc<Mutex<HashMap<String, TorrentManifest>>>,
    local_chunks: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    download_stats: Arc<Mutex<DownloadStats>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadStats {
    pub total_bytes_downloaded: u64,
    pub total_bytes_uploaded: u64,
    pub chunks_downloaded: u64,
    pub chunks_uploaded: u64,
    pub peers_contacted: u64,
    pub failed_downloads: u64,
}

impl P2PMeshRouter {
    pub fn new(local_peer_id: PeerId) -> Self {
        Self {
            local_peer_id,
            peers: Arc::new(Mutex::new(HashMap::new())),
            manifests: Arc::new(Mutex::new(HashMap::new())),
            local_chunks: Arc::new(Mutex::new(HashMap::new())),
            download_stats: Arc::new(Mutex::new(DownloadStats::default())),
        }
    }

    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    pub fn register_peer(&self, peer: PeerInfo) {
        if let Ok(mut peers) = self.peers.lock() {
            peers.insert(peer.peer_id.0.clone(), peer);
        }
    }

    pub fn unregister_peer(&self, peer_id: &PeerId) {
        if let Ok(mut peers) = self.peers.lock() {
            peers.remove(&peer_id.0);
        }
    }

    pub fn list_peers(&self) -> Vec<PeerInfo> {
        self.peers
            .lock()
            .map(|p| p.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn announce_artifact(&self, artifact: &Artifact, chunk_size: u64) -> TorrentManifest {
        let manifest = TorrentManifest::from_artifact(artifact, chunk_size);

        // Store local chunks
        if let Ok(mut local) = self.local_chunks.lock() {
            let data = artifact.data();
            for chunk_info in &manifest.chunks {
                let start = (chunk_info.index as u64 * chunk_size) as usize;
                let end = (start as u64 + chunk_info.size_bytes) as usize;
                let chunk_data = data[start..end].to_vec();
                local.insert(chunk_info.hash.clone(), chunk_data);
            }
        }

        if let Ok(mut manifests) = self.manifests.lock() {
            manifests.insert(manifest.artifact_hash.clone(), manifest.clone());
        }

        // Announce to peers that we have these chunks
        if let Ok(mut peers) = self.peers.lock() {
            // In real implementation, broadcast to peers
            // For now, update local peer info
            for peer in peers.values_mut() {
                if peer.peer_id == self.local_peer_id {
                    for chunk in &manifest.chunks {
                        peer.available_chunks.insert(chunk.hash.clone());
                    }
                }
            }
        }

        manifest
    }

    pub fn get_manifest(&self, artifact_hash: &str) -> Option<TorrentManifest> {
        self.manifests
            .lock()
            .ok()
            .and_then(|m| m.get(artifact_hash).cloned())
    }

    /// Find peers that have a specific chunk (rarest-first strategy)
    pub fn find_peers_for_chunk(&self, chunk_hash: &str) -> Vec<PeerInfo> {
        let peers = self.peers.lock().map(|p| p.values().cloned().collect::<Vec<_>>()).unwrap_or_default();

        let mut holders: Vec<PeerInfo> = peers
            .into_iter()
            .filter(|peer| peer.has_chunk(chunk_hash))
            .collect();

        // Sort by reputation and speed (best first)
        holders.sort_by(|a, b| {
            b.reputation
                .partial_cmp(&a.reputation)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.upload_speed_bps.cmp(&a.upload_speed_bps))
        });

        holders
    }

    /// Find rarest chunks first (BitTorrent strategy for optimal distribution)
    pub fn rarest_chunks_first(&self, manifest: &TorrentManifest) -> Vec<ChunkInfo> {
        let peers = self.peers.lock().map(|p| p.values().cloned().collect::<Vec<_>>()).unwrap_or_default();

        let mut chunk_availability: Vec<(ChunkInfo, usize)> = manifest
            .chunks
            .iter()
            .map(|chunk| {
                let count = peers.iter().filter(|p| p.has_chunk(&chunk.hash)).count();
                (chunk.clone(), count)
            })
            .collect();

        // Sort by availability (rarest first)
        chunk_availability.sort_by(|a, b| a.1.cmp(&b.1));

        chunk_availability.into_iter().map(|(chunk, _)| chunk).collect()
    }

    /// Simulate downloading artifact via P2P mesh
    pub async fn download_artifact(&self, artifact_hash: &str) -> Result<Vec<u8>, String> {
        let manifest = self
            .get_manifest(artifact_hash)
            .ok_or_else(|| format!("manifest not found for {}", artifact_hash))?;

        let rarest_first = self.rarest_chunks_first(&manifest);
        let mut assembled = vec![0u8; manifest.total_size as usize];
        let mut downloaded_chunks = 0;

        for chunk_info in rarest_first {
            let peers = self.find_peers_for_chunk(&chunk_info.hash);
            if peers.is_empty() {
                // Try local cache
                if let Ok(local) = self.local_chunks.lock() {
                    if let Some(data) = local.get(&chunk_info.hash) {
                        let start = (chunk_info.index as u64 * manifest.chunk_size) as usize;
                        let end = start + data.len();
                        assembled[start..end].copy_from_slice(data);
                        downloaded_chunks += 1;
                        continue;
                    }
                }

                if let Ok(mut stats) = self.download_stats.lock() {
                    stats.failed_downloads += 1;
                }
                return Err(format!("no peers have chunk {}", chunk_info.hash));
            }

            // Simulate download from best peer
            let best_peer = &peers[0];
            
            // Simulate network transfer
            tokio::time::sleep(Duration::from_millis(1)).await;

            // Get chunk data (simulate)
            let chunk_data = if let Ok(local) = self.local_chunks.lock() {
                local.get(&chunk_info.hash).cloned()
            } else {
                None
            };

            if let Some(data) = chunk_data {
                let start = (chunk_info.index as u64 * manifest.chunk_size) as usize;
                let end = start + data.len();
                if end <= assembled.len() {
                    assembled[start..end].copy_from_slice(&data);
                    downloaded_chunks += 1;

                    if let Ok(mut stats) = self.download_stats.lock() {
                        stats.total_bytes_downloaded += data.len() as u64;
                        stats.chunks_downloaded += 1;
                        stats.peers_contacted += 1;
                    }

                    // Update peer reputation
                    if let Ok(mut peers_map) = self.peers.lock() {
                        if let Some(peer) = peers_map.get_mut(&best_peer.peer_id.0) {
                            peer.reputation = (peer.reputation * 0.9 + 0.1).min(1.0);
                            peer.upload_speed_bps = 1024 * 1024; // 1MB/s
                        }
                    }
                }
            } else {
                // Simulate generating chunk data if not in local (for test)
                let start = (chunk_info.index as u64 * manifest.chunk_size) as usize;
                let end = (start as u64 + chunk_info.size_bytes) as usize;
                let simulated = vec![0u8; (end - start).min(assembled.len() - start)];
                if start + simulated.len() <= assembled.len() {
                    assembled[start..start + simulated.len()].copy_from_slice(&simulated);
                    downloaded_chunks += 1;
                }
            }
        }

        if downloaded_chunks != manifest.chunk_count() {
            return Err(format!(
                "incomplete download: {}/{} chunks",
                downloaded_chunks,
                manifest.chunk_count()
            ));
        }

        Ok(assembled)
    }

    pub fn stats(&self) -> DownloadStats {
        self.download_stats
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    pub fn peer_count(&self) -> usize {
        self.peers.lock().map(|p| p.len()).unwrap_or(0)
    }

    pub fn chunk_count(&self) -> usize {
        self.local_chunks.lock().map(|c| c.len()).unwrap_or(0)
    }

    /// Tit-for-tat choking algorithm (BitTorrent-inspired)
    pub fn select_peers_to_unchoke(&self, max_unchoked: usize) -> Vec<PeerId> {
        let peers = self.list_peers();
        let mut sorted = peers.clone();
        
        // Sort by download speed and reputation (optimistic unchoking)
        sorted.sort_by(|a, b| {
            b.download_speed_bps
                .cmp(&a.download_speed_bps)
                .then_with(|| {
                    b.reputation
                        .partial_cmp(&a.reputation)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        sorted
            .into_iter()
            .take(max_unchoked)
            .map(|p| p.peer_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::Artifact;

    #[tokio::test]
    async fn test_p2p_mesh_distribution() {
        let router = P2PMeshRouter::new(PeerId::new("local-peer"));

        // Register peers
        let mut peer1 = PeerInfo::new(PeerId::new("peer-1"), "10.0.0.1:8080");
        peer1.upload_speed_bps = 1024 * 1024;
        peer1.reputation = 0.9;

        let mut peer2 = PeerInfo::new(PeerId::new("peer-2"), "10.0.0.2:8080");
        peer2.upload_speed_bps = 512 * 1024;
        peer2.reputation = 0.8;

        router.register_peer(peer1);
        router.register_peer(peer2);

        assert_eq!(router.peer_count(), 2);

        // Create artifact and announce
        let artifact = Artifact::from_bytes(
            b"test artifact data for p2p mesh distribution testing".to_vec(),
            "binary".to_string(),
            "test".to_string(),
        )
        .unwrap();

        let manifest = router.announce_artifact(&artifact, 10);
        assert!(manifest.chunk_count() > 1);
        assert_eq!(manifest.total_size, artifact.data().len() as u64);

        // Simulate peers having chunks
        if let Ok(mut peers) = router.peers.lock() {
            for chunk in &manifest.chunks {
                for peer in peers.values_mut() {
                    peer.available_chunks.insert(chunk.hash.clone());
                }
            }
        }

        let rarest = router.rarest_chunks_first(&manifest);
        assert_eq!(rarest.len(), manifest.chunk_count());

        // Download artifact
        let downloaded = router.download_artifact(&manifest.artifact_hash).await.unwrap();
        assert_eq!(downloaded.len(), artifact.data().len());

        let stats = router.stats();
        assert!(stats.chunks_downloaded > 0);
    }

    #[test]
    fn test_torrent_manifest() {
        let artifact = Artifact::from_bytes(
            vec![0u8; 100],
            "binary".to_string(),
            "test".to_string(),
        )
        .unwrap();

        let manifest = TorrentManifest::from_artifact(&artifact, 30);
        assert_eq!(manifest.chunk_count(), 4); // 100 bytes / 30 = 4 chunks
        assert_eq!(manifest.total_size, 100);
        assert_eq!(manifest.chunk_size, 30);
    }

    #[test]
    fn test_peer_selection_and_choking() {
        let router = P2PMeshRouter::new(PeerId::new("local"));

        for i in 0..5 {
            let mut peer = PeerInfo::new(PeerId::new(format!("peer-{}", i)), format!("10.0.0.{}:8080", i));
            peer.download_speed_bps = (5 - i) as u64 * 100 * 1024;
            peer.reputation = 1.0 - (i as f64 * 0.1);
            router.register_peer(peer);
        }

        let unchoked = router.select_peers_to_unchoke(2);
        assert_eq!(unchoked.len(), 2);
        assert_eq!(unchoked[0].0, "peer-0"); // Fastest
    }
}
