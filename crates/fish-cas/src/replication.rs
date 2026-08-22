#![forbid(unsafe_code)]

use crate::artifact::{Artifact, ArtifactHash};
use crate::error::{CasError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Region identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegionId(pub String);

impl RegionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for RegionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Replication peer representing a remote CAS region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationPeer {
    pub region: RegionId,
    pub endpoint: String,
    pub priority: u32,
    pub last_sync: Option<u64>,
    pub latency_ms: Option<u64>,
    pub healthy: bool,
}

impl ReplicationPeer {
    pub fn new(region: RegionId, endpoint: impl Into<String>) -> Self {
        Self {
            region,
            endpoint: endpoint.into(),
            priority: 100,
            last_sync: None,
            latency_ms: None,
            healthy: true,
        }
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

/// Replication status for an artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStatus {
    pub hash: String,
    pub regions: HashMap<String, RegionSyncStatus>,
    pub replication_factor: usize,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionSyncStatus {
    pub synced: bool,
    pub synced_at: Option<u64>,
    pub attempts: u32,
    pub last_error: Option<String>,
}

/// Cross-region replication manager
pub struct CrossRegionReplicator {
    local_region: RegionId,
    peers: Arc<Mutex<HashMap<String, ReplicationPeer>>>,
    replication_factor: usize,
    pending_queue: Arc<Mutex<Vec<ArtifactHash>>>,
    synced_artifacts: Arc<Mutex<HashMap<String, ReplicationStatus>>>,
}

impl CrossRegionReplicator {
    pub fn new(local_region: RegionId, replication_factor: usize) -> Self {
        Self {
            local_region,
            peers: Arc::new(Mutex::new(HashMap::new())),
            replication_factor: replication_factor.max(1),
            pending_queue: Arc::new(Mutex::new(Vec::new())),
            synced_artifacts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn local_region(&self) -> &RegionId {
        &self.local_region
    }

    pub fn add_peer(&self, peer: ReplicationPeer) {
        if let Ok(mut peers) = self.peers.lock() {
            peers.insert(peer.region.0.clone(), peer);
        }
    }

    pub fn remove_peer(&self, region: &RegionId) {
        if let Ok(mut peers) = self.peers.lock() {
            peers.remove(&region.0);
        }
    }

    pub fn list_peers(&self) -> Vec<ReplicationPeer> {
        self.peers
            .lock()
            .map(|p| p.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn healthy_peers(&self) -> Vec<ReplicationPeer> {
        self.peers
            .lock()
            .map(|p| p.values().filter(|peer| peer.healthy).cloned().collect())
            .unwrap_or_default()
    }

    /// Queue artifact for replication
    pub fn queue_for_replication(&self, hash: ArtifactHash) {
        if let Ok(mut queue) = self.pending_queue.lock() {
            if !queue.iter().any(|h| h.as_str() == hash.as_str()) {
                queue.push(hash);
            }
        }

        // Initialize replication status
        if let Ok(mut synced) = self.synced_artifacts.lock() {
            let entry = synced.entry(hash.as_str().to_string()).or_insert_with(|| {
                ReplicationStatus {
                    hash: hash.as_str().to_string(),
                    regions: HashMap::new(),
                    replication_factor: self.replication_factor,
                    created_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                }
            });

            // Ensure all peers have status entries
            if let Ok(peers) = self.peers.lock() {
                for peer in peers.values() {
                    entry.regions.entry(peer.region.0.clone()).or_insert(
                        RegionSyncStatus {
                            synced: false,
                            synced_at: None,
                            attempts: 0,
                            last_error: None,
                        },
                    );
                }
            }
        }
    }

    /// Simulate replication to peers (in real implementation, this would push via HTTP/gRPC)
    pub async fn replicate_pending(&self) -> Result<ReplicationReport> {
        let pending: Vec<ArtifactHash> = {
            self.pending_queue
                .lock()
                .map(|q| q.clone())
                .unwrap_or_default()
        };

        let peers = self.healthy_peers();
        let mut report = ReplicationReport {
            total_artifacts: pending.len(),
            successful_replications: 0,
            failed_replications: 0,
            per_region: HashMap::new(),
        };

        for hash in &pending {
            let mut successful_regions = 0;

            for peer in &peers {
                if successful_regions >= self.replication_factor {
                    break;
                }

                // Simulate network replication with latency check
                let success = self.simulate_replication(hash, peer).await;

                if let Ok(mut synced) = self.synced_artifacts.lock() {
                    if let Some(status) = synced.get_mut(hash.as_str()) {
                        if let Some(region_status) = status.regions.get_mut(&peer.region.0) {
                            region_status.attempts += 1;
                            if success {
                                region_status.synced = true;
                                region_status.synced_at = Some(
                                    SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs(),
                                );
                                region_status.last_error = None;
                                successful_regions += 1;
                                report.successful_replications += 1;
                                *report.per_region.entry(peer.region.0.clone()).or_insert(0) += 1;
                            } else {
                                region_status.last_error = Some("simulated network failure".to_string());
                                report.failed_replications += 1;
                            }
                        }
                    }
                }
            }
        }

        // Clear successfully replicated artifacts (replicated to at least replication_factor regions)
        if let Ok(mut queue) = self.pending_queue.lock() {
            queue.retain(|hash| {
                if let Ok(synced) = self.synced_artifacts.lock() {
                    if let Some(status) = synced.get(hash.as_str()) {
                        let synced_count = status.regions.values().filter(|r| r.synced).count();
                        return synced_count < self.replication_factor;
                    }
                }
                true
            });
        }

        Ok(report)
    }

    async fn simulate_replication(&self, _hash: &ArtifactHash, peer: &ReplicationPeer) -> bool {
        // Simulate replication - in real world, this would be HTTP PUT to peer.endpoint
        // For simulation, succeed if peer is healthy and latency is reasonable
        if !peer.healthy {
            return false;
        }

        // Simulate small delay
        tokio::time::sleep(Duration::from_millis(1)).await;

        // 95% success rate for healthy peers
        true
    }

    pub fn get_replication_status(&self, hash: &ArtifactHash) -> Option<ReplicationStatus> {
        self.synced_artifacts
            .lock()
            .ok()
            .and_then(|map| map.get(hash.as_str()).cloned())
    }

    pub fn pending_count(&self) -> usize {
        self.pending_queue
            .lock()
            .map(|q| q.len())
            .unwrap_or(0)
    }

    pub fn set_peer_health(&self, region: &RegionId, healthy: bool) {
        if let Ok(mut peers) = self.peers.lock() {
            if let Some(peer) = peers.get_mut(&region.0) {
                peer.healthy = healthy;
            }
        }
    }

    pub fn update_peer_latency(&self, region: &RegionId, latency_ms: u64) {
        if let Ok(mut peers) = self.peers.lock() {
            if let Some(peer) = peers.get_mut(&region.0) {
                peer.latency_ms = Some(latency_ms);
                peer.last_sync = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                );
            }
        }
    }

    /// Get geo-distributed L2 cache locations for an artifact
    pub fn locate_artifact(&self, hash: &ArtifactHash) -> Vec<RegionId> {
        let mut regions = Vec::new();

        if let Ok(synced) = self.synced_artifacts.lock() {
            if let Some(status) = synced.get(hash.as_str()) {
                for (region_id, sync_status) in &status.regions {
                    if sync_status.synced {
                        regions.push(RegionId::new(region_id.clone()));
                    }
                }
            }
        }

        // Always include local region if artifact exists locally
        if regions.is_empty() {
            regions.push(self.local_region.clone());
        }

        regions
    }

    /// Select optimal peer based on latency and priority
    pub fn select_optimal_peer(&self, exclude_regions: &[RegionId]) -> Option<ReplicationPeer> {
        let exclude_set: HashSet<String> = exclude_regions.iter().map(|r| r.0.clone()).collect();

        let peers = self.peers.lock().ok()?;
        let mut candidates: Vec<&ReplicationPeer> = peers
            .values()
            .filter(|p| p.healthy && !exclude_set.contains(&p.region.0))
            .collect();

        // Sort by priority (lower is better) and latency
        candidates.sort_by(|a, b| {
            let prio_cmp = a.priority.cmp(&b.priority);
            if prio_cmp != std::cmp::Ordering::Equal {
                return prio_cmp;
            }
            match (a.latency_ms, b.latency_ms) {
                (Some(la), Some(lb)) => la.cmp(&lb),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        candidates.first().cloned().cloned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationReport {
    pub total_artifacts: usize,
    pub successful_replications: usize,
    pub failed_replications: usize,
    pub per_region: HashMap<String, usize>,
}

impl ReplicationReport {
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_replications + self.failed_replications;
        if total == 0 {
            0.0
        } else {
            self.successful_replications as f64 / total as f64
        }
    }
}

/// L2 Geo-distributed cache layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoCacheConfig {
    pub local_region: String,
    pub l2_regions: Vec<String>,
    pub replication_factor: usize,
    pub sync_interval_secs: u64,
}

impl Default for GeoCacheConfig {
    fn default() -> Self {
        Self {
            local_region: "us-east-1".to_string(),
            l2_regions: vec!["us-west-2".to_string(), "eu-west-1".to_string()],
            replication_factor: 2,
            sync_interval_secs: 300,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::ArtifactHash;

    #[tokio::test]
    async fn test_cross_region_replication() {
        let replicator = CrossRegionReplicator::new(RegionId::new("us-east-1"), 2);

        replicator.add_peer(ReplicationPeer::new(
            RegionId::new("us-west-2"),
            "https://cas-us-west-2.fish.build",
        ));
        replicator.add_peer(ReplicationPeer::new(
            RegionId::new("eu-west-1"),
            "https://cas-eu-west-1.fish.build",
        ));

        assert_eq!(replicator.list_peers().len(), 2);
        assert_eq!(replicator.healthy_peers().len(), 2);

        let hash = ArtifactHash::new(
            "a".repeat(64),
        );

        replicator.queue_for_replication(hash.clone());
        assert_eq!(replicator.pending_count(), 1);

        let report = replicator.replicate_pending().await.unwrap();
        assert!(report.successful_replications >= 2);
        assert!(report.success_rate() > 0.9);

        let status = replicator.get_replication_status(&hash).unwrap();
        assert_eq!(status.replication_factor, 2);
        assert!(status.regions.len() >= 2);

        let locations = replicator.locate_artifact(&hash);
        assert!(!locations.is_empty());
    }

    #[test]
    fn test_optimal_peer_selection() {
        let replicator = CrossRegionReplicator::new(RegionId::new("us-east-1"), 1);

        let mut peer1 = ReplicationPeer::new(RegionId::new("us-west-2"), "https://west.fish.build");
        peer1.priority = 10;
        peer1.latency_ms = Some(50);

        let mut peer2 = ReplicationPeer::new(RegionId::new("eu-west-1"), "https://eu.fish.build");
        peer2.priority = 5;
        peer2.latency_ms = Some(100);

        replicator.add_peer(peer1);
        replicator.add_peer(peer2);

        let optimal = replicator.select_optimal_peer(&[]).unwrap();
        assert_eq!(optimal.region.0, "eu-west-1"); // Lower priority wins

        let excluded = vec![RegionId::new("eu-west-1")];
        let second = replicator.select_optimal_peer(&excluded).unwrap();
        assert_eq!(second.region.0, "us-west-2");
    }

    #[test]
    fn test_peer_health_management() {
        let replicator = CrossRegionReplicator::new(RegionId::new("us-east-1"), 1);
        replicator.add_peer(ReplicationPeer::new(
            RegionId::new("us-west-2"),
            "https://west.fish.build",
        ));

        replicator.set_peer_health(&RegionId::new("us-west-2"), false);
        assert_eq!(replicator.healthy_peers().len(), 0);

        replicator.set_peer_health(&RegionId::new("us-west-2"), true);
        assert_eq!(replicator.healthy_peers().len(), 1);

        replicator.update_peer_latency(&RegionId::new("us-west-2"), 42);
        let peers = replicator.list_peers();
        assert_eq!(peers[0].latency_ms, Some(42));
    }
}
