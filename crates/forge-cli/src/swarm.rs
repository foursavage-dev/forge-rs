#![allow(dead_code)]

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub const SWARM_DISCOVERY_PORT: u16 = 7890;

#[derive(Debug, Clone)]
pub struct SwarmPeer {
    pub peer_id: String,
    pub address: SocketAddr,
    pub compute_port: Option<u16>,
    pub concurrency: usize,
    pub last_seen: Instant,
}

#[derive(Debug, Clone)]
pub struct SwarmCache {
    peer_id: String,
    peers: Arc<RwLock<HashMap<String, SwarmPeer>>>,
    socket: Option<Arc<UdpSocket>>,
    enabled: bool,
}

impl SwarmCache {
    pub fn new(enabled: bool) -> Self {
        let peer_id = format!(
            "peer_{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        let socket = if enabled {
            let sock = UdpSocket::bind("0.0.0.0:0").ok();
            if let Some(ref s) = sock {
                let _ = s.set_broadcast(true);
                let _ = s.set_read_timeout(Some(Duration::from_millis(50)));
            }
            sock.map(Arc::new)
        } else {
            None
        };

        Self {
            peer_id,
            peers: Arc::new(RwLock::new(HashMap::new())),
            socket,
            enabled,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn broadcast_presence(&self, service_port: u16) -> std::io::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(ref socket) = self.socket {
            let message = format!("FORGE_SWARM_ANNOUNCE:{}:{}", self.peer_id, service_port);
            let broadcast_addr = SocketAddr::from(([255, 255, 255, 255], SWARM_DISCOVERY_PORT));
            socket.send_to(message.as_bytes(), broadcast_addr)?;
        }

        Ok(())
    }

    pub fn broadcast_compute_worker(&self, worker_port: u16, concurrency: usize) -> std::io::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(ref socket) = self.socket {
            let message = format!(
                "FORGE_SWARM_COMPUTE:{}:{}:{}",
                self.peer_id, worker_port, concurrency
            );
            let broadcast_addr = SocketAddr::from(([255, 255, 255, 255], SWARM_DISCOVERY_PORT));
            socket.send_to(message.as_bytes(), broadcast_addr)?;
        }

        Ok(())
    }

    pub fn poll_peers(&self) {
        if !self.enabled {
            return;
        }

        if let Some(ref socket) = self.socket {
            let mut buf = [0u8; 1024];
            while let Ok((amt, src)) = socket.recv_from(&mut buf) {
                if let Ok(msg) = std::str::from_utf8(&buf[..amt]) {
                    if let Some(payload) = msg.strip_prefix("FORGE_SWARM_ANNOUNCE:") {
                        let parts: Vec<&str> = payload.split(':').collect();
                        if parts.len() == 2 {
                            let peer_id = parts[0].to_string();
                            if peer_id != self.peer_id {
                                if let Ok(port) = parts[1].parse::<u16>() {
                                    let mut peer_addr = src;
                                    peer_addr.set_port(port);
                                    if let Ok(mut peers) = self.peers.write() {
                                        peers.insert(
                                            peer_id.clone(),
                                            SwarmPeer {
                                                peer_id,
                                                address: peer_addr,
                                                compute_port: None,
                                                concurrency: 0,
                                                last_seen: Instant::now(),
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    } else if let Some(payload) = msg.strip_prefix("FORGE_SWARM_COMPUTE:") {
                        let parts: Vec<&str> = payload.split(':').collect();
                        if parts.len() == 3 {
                            let peer_id = parts[0].to_string();
                            if peer_id != self.peer_id {
                                if let Ok(compute_port) = parts[1].parse::<u16>() {
                                    let concurrency = parts[2].parse::<usize>().unwrap_or(4);
                                    let mut peer_addr = src;
                                    peer_addr.set_port(compute_port);
                                    if let Ok(mut peers) = self.peers.write() {
                                        peers.insert(
                                            peer_id.clone(),
                                            SwarmPeer {
                                                peer_id,
                                                address: peer_addr,
                                                compute_port: Some(compute_port),
                                                concurrency,
                                                last_seen: Instant::now(),
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn active_peer_count(&self) -> usize {
        self.poll_peers();
        if let Ok(peers) = self.peers.read() {
            let now = Instant::now();
            peers
                .values()
                .filter(|p| now.duration_since(p.last_seen) < Duration::from_secs(60))
                .count()
        } else {
            0
        }
    }

    pub fn discovered_compute_endpoints(&self) -> Vec<String> {
        self.poll_peers();
        if let Ok(peers) = self.peers.read() {
            let now = Instant::now();
            peers
                .values()
                .filter(|p| {
                    p.compute_port.is_some()
                        && now.duration_since(p.last_seen) < Duration::from_secs(60)
                })
                .map(|p| format!("http://{}", p.address))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn list_peers(&self) -> Vec<SwarmPeer> {
        self.poll_peers();
        if let Ok(peers) = self.peers.read() {
            peers.values().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_cache_disabled() {
        let swarm = SwarmCache::new(false);
        assert!(!swarm.is_enabled());
        assert_eq!(swarm.active_peer_count(), 0);
        assert!(swarm.list_peers().is_empty());
        assert!(swarm.discovered_compute_endpoints().is_empty());
    }

    #[test]
    fn test_swarm_cache_enabled_initial_state() {
        let swarm = SwarmCache::new(true);
        assert!(swarm.is_enabled());
        assert_eq!(swarm.active_peer_count(), 0);
        let _ = swarm.broadcast_presence(7890);
        let _ = swarm.broadcast_compute_worker(7891, 8);
    }
}
