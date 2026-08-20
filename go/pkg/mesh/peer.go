package mesh

import (
	"errors"
	"sync"
	"time"
)

type CASChunk struct {
	Digest    string    `json:"digest"`
	SizeBytes int64     `json:"size_bytes"`
	OwnerPeer string    `json:"owner_peer"`
	CreatedAt time.Time `json:"created_at"`
}

type P2PMeshRouter struct {
	mu     sync.RWMutex
	chunks map[string][]CASChunk
	peers  map[string]string
}

func NewP2PMeshRouter() *P2PMeshRouter {
	return &P2PMeshRouter{
		chunks: make(map[string][]CASChunk),
		peers:  make(map[string]string),
	}
}

func (r *P2PMeshRouter) RegisterPeer(peerID string, address string) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.peers[peerID] = address
}

func (r *P2PMeshRouter) AnnounceChunk(chunk CASChunk) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.chunks[chunk.Digest] = append(r.chunks[chunk.Digest], chunk)
}

func (r *P2PMeshRouter) LocateChunk(digest string) (CASChunk, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	providers, ok := r.chunks[digest]
	if !ok || len(providers) == 0 {
		return CASChunk{}, errors.New("chunk not found in p2p mesh")
	}
	return providers[0], nil
}
