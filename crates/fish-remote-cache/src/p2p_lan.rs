use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct LanPeerNode {
    pub peer_id: String,
    pub address: SocketAddr,
    pub available_artifacts: HashSet<String>,
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
            },
        );
    }

    pub fn announce_artifact(&self, peer_id: &str, artifact_hash: &str) {
        let mut map = self.peers.write().unwrap();
        if let Some(peer) = map.get_mut(peer_id) {
            peer.available_artifacts.insert(artifact_hash.to_string());
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

    pub fn peer_count(&self) -> usize {
        self.peers.read().unwrap().len()
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
}
