package mesh

import (
	"errors"
	"sort"
	"sync"
	"time"
)

type CASChunk struct {
	Digest    string    `json:"digest"`
	SizeBytes int64     `json:"size_bytes"`
	OwnerPeer string    `json:"owner_peer"`
	CreatedAt time.Time `json:"created_at"`
	Region    string    `json:"region"`
	Priority  int       `json:"priority"`
}

type PeerMetrics struct {
	UploadBytes   int64   `json:"upload_bytes"`
	DownloadBytes int64   `json:"download_bytes"`
	ChunksServed  int64   `json:"chunks_served"`
	LatencyMs     float64 `json:"latency_ms"`
	Reputation    float64 `json:"reputation"`
	LastSeen      time.Time `json:"last_seen"`
}

type PeerInfo struct {
	PeerID      string            `json:"peer_id"`
	Address     string            `json:"address"`
	Region      string            `json:"region"`
	Metrics     PeerMetrics       `json:"metrics"`
	Healthy     bool              `json:"healthy"`
	Tags        map[string]string `json:"tags"`
}

type P2PMeshRouter struct {
	mu            sync.RWMutex
	chunks        map[string][]CASChunk
	peers         map[string]*PeerInfo
	downloadStats map[string]int64
	replicationFactor int
}

func NewP2PMeshRouter() *P2PMeshRouter {
	return &P2PMeshRouter{
		chunks:            make(map[string][]CASChunk),
		peers:             make(map[string]*PeerInfo),
		downloadStats:     make(map[string]int64),
		replicationFactor: 3,
	}
}

func NewP2PMeshRouterWithReplication(factor int) *P2PMeshRouter {
	router := NewP2PMeshRouter()
	router.replicationFactor = factor
	return router
}

func (r *P2PMeshRouter) RegisterPeer(peerID string, address string) {
	r.mu.Lock()
	defer r.mu.Unlock()
	
	if existing, ok := r.peers[peerID]; ok {
		existing.Address = address
		existing.Metrics.LastSeen = time.Now()
		existing.Healthy = true
	} else {
		r.peers[peerID] = &PeerInfo{
			PeerID:  peerID,
			Address: address,
			Metrics: PeerMetrics{
				Reputation: 1.0,
				LastSeen:   time.Now(),
			},
			Healthy: true,
			Tags:    make(map[string]string),
		}
	}
}

func (r *P2PMeshRouter) RegisterPeerWithRegion(peerID string, address string, region string) {
	r.RegisterPeer(peerID, address)
	r.mu.Lock()
	defer r.mu.Unlock()
	if peer, ok := r.peers[peerID]; ok {
		peer.Region = region
	}
}

func (r *P2PMeshRouter) UnregisterPeer(peerID string) {
	r.mu.Lock()
	defer r.mu.Unlock()
	delete(r.peers, peerID)
}

func (r *P2PMeshRouter) SetPeerHealth(peerID string, healthy bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	if peer, ok := r.peers[peerID]; ok {
		peer.Healthy = healthy
	}
}

func (r *P2PMeshRouter) AnnounceChunk(chunk CASChunk) {
	r.mu.Lock()
	defer r.mu.Unlock()
	
	if chunk.CreatedAt.IsZero() {
		chunk.CreatedAt = time.Now()
	}
	
	r.chunks[chunk.Digest] = append(r.chunks[chunk.Digest], chunk)
	
	// Update peer metrics
	if peer, ok := r.peers[chunk.OwnerPeer]; ok {
		peer.Metrics.ChunksServed++
		peer.Metrics.UploadBytes += chunk.SizeBytes
	}
}

func (r *P2PMeshRouter) LocateChunk(digest string) (CASChunk, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	providers, ok := r.chunks[digest]
	if !ok || len(providers) == 0 {
		return CASChunk{}, errors.New("chunk not found in p2p mesh")
	}
	
	// Return best provider based on reputation and latency
	best := providers[0]
	bestScore := -1.0
	
	for _, chunk := range providers {
		if peer, ok := r.peers[chunk.OwnerPeer]; ok && peer.Healthy {
			score := peer.Metrics.Reputation * 100 - peer.Metrics.LatencyMs
			if score > bestScore {
				bestScore = score
				best = chunk
			}
		}
	}
	
	return best, nil
}

func (r *P2PMeshRouter) LocateAllProviders(digest string) ([]CASChunk, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	providers, ok := r.chunks[digest]
	if !ok || len(providers) == 0 {
		return nil, errors.New("chunk not found in p2p mesh")
	}
	
	// Sort by peer quality
	sort.Slice(providers, func(i, j int) bool {
		peerI, okI := r.peers[providers[i].OwnerPeer]
		peerJ, okJ := r.peers[providers[j].OwnerPeer]
		if !okI || !okJ {
			return providers[i].CreatedAt.After(providers[j].CreatedAt)
		}
		return peerI.Metrics.Reputation > peerJ.Metrics.Reputation
	})
	
	return providers, nil
}

// BitTorrent-inspired: find rarest chunks for optimal distribution
func (r *P2PMeshRouter) FindRarestChunks(limit int) []string {
	r.mu.RLock()
	defer r.mu.RUnlock()
	
	type chunkCount struct {
		digest string
		count  int
	}
	
	var counts []chunkCount
	for digest, providers := range r.chunks {
		counts = append(counts, chunkCount{digest: digest, count: len(providers)})
	}
	
	sort.Slice(counts, func(i, j int) bool {
		return counts[i].count < counts[j].count
	})
	
	var rarest []string
	for i := 0; i < len(counts) && i < limit; i++ {
		rarest = append(rarest, counts[i].digest)
	}
	
	return rarest
}

// Tit-for-tat: select peers to unchoke based on reciprocation
func (r *P2PMeshRouter) SelectPeersToUnchoke(maxUnchoked int) []string {
	r.mu.RLock()
	defer r.mu.RUnlock()
	
	type peerScore struct {
		peerID string
		score  float64
	}
	
	var scores []peerScore
	for peerID, info := range r.peers {
		if !info.Healthy {
			continue
		}
		// Score based on download bytes (reciprocation) and reputation
		score := float64(info.Metrics.DownloadBytes)*0.5 + info.Metrics.Reputation*100
		scores = append(scores, peerScore{peerID: peerID, score: score})
	}
	
	sort.Slice(scores, func(i, j int) bool {
		return scores[i].score > scores[j].score
	})
	
	var unchoked []string
	for i := 0; i < len(scores) && i < maxUnchoked; i++ {
		unchoked = append(unchoked, scores[i].peerID)
	}
	
	return unchoked
}

func (r *P2PMeshRouter) GetPeerCount() int {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return len(r.peers)
}

func (r *P2PMeshRouter) GetChunkCount() int {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return len(r.chunks)
}

func (r *P2PMeshRouter) GetHealthyPeerCount() int {
	r.mu.RLock()
	defer r.mu.RUnlock()
	count := 0
	for _, peer := range r.peers {
		if peer.Healthy {
			count++
		}
	}
	return count
}

// Replicate chunk to achieve desired replication factor
func (r *P2PMeshRouter) EnsureReplication(digest string) []string {
	r.mu.Lock()
	defer r.mu.Unlock()
	
	providers, ok := r.chunks[digest]
	if !ok {
		return nil
	}
	
	currentReplicas := len(providers)
	needed := r.replicationFactor - currentReplicas
	if needed <= 0 {
		return nil
	}
	
	// Find peers that don't have this chunk
	var candidates []*PeerInfo
	for _, peer := range r.peers {
		if !peer.Healthy {
			continue
		}
		hasChunk := false
		for _, p := range providers {
			if p.OwnerPeer == peer.PeerID {
				hasChunk = true
				break
			}
		}
		if !hasChunk {
			candidates = append(candidates, peer)
		}
	}
	
	// Sort candidates by reputation
	sort.Slice(candidates, func(i, j int) bool {
		return candidates[i].Metrics.Reputation > candidates[j].Metrics.Reputation
	})
	
	var replicatedTo []string
	for i := 0; i < len(candidates) && i < needed; i++ {
		// Simulate replication
		newChunk := CASChunk{
			Digest:    digest,
			SizeBytes: providers[0].SizeBytes,
			OwnerPeer: candidates[i].PeerID,
			CreatedAt: time.Now(),
			Region:    candidates[i].Region,
		}
		r.chunks[digest] = append(r.chunks[digest], newChunk)
		replicatedTo = append(replicatedTo, candidates[i].PeerID)
	}
	
	return replicatedTo
}

func (r *P2PMeshRouter) GetStats() map[string]interface{} {
	r.mu.RLock()
	defer r.mu.RUnlock()
	
	totalChunks := 0
	for _, providers := range r.chunks {
		totalChunks += len(providers)
	}
	
	healthyPeers := 0
	for _, peer := range r.peers {
		if peer.Healthy {
			healthyPeers++
		}
	}
	
	return map[string]interface{}{
		"total_peers":        len(r.peers),
		"healthy_peers":      healthyPeers,
		"unique_chunks":      len(r.chunks),
		"total_replicas":     totalChunks,
		"replication_factor": r.replicationFactor,
	}
}
