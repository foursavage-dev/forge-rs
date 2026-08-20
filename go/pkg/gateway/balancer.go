package gateway

import (
	"errors"
	"sync"
	"sync/atomic"
)

type WorkerTarget struct {
	ID        string
	Address   string
	ActiveOps int64
	Healthy   bool
}

type LoadBalancer struct {
	mu      sync.RWMutex
	targets []*WorkerTarget
	counter uint64
}

func NewLoadBalancer() *LoadBalancer {
	return &LoadBalancer{
		targets: make([]*WorkerTarget, 0),
	}
}

func (lb *LoadBalancer) AddTarget(id, address string) {
	lb.mu.Lock()
	defer lb.mu.Unlock()
	lb.targets = append(lb.targets, &WorkerTarget{
		ID:      id,
		Address: address,
		Healthy: true,
	})
}

func (lb *LoadBalancer) SelectLeastLoaded() (*WorkerTarget, error) {
	lb.mu.RLock()
	defer lb.mu.RUnlock()

	var best *WorkerTarget
	for _, t := range lb.targets {
		if !t.Healthy {
			continue
		}
		if best == nil || atomic.LoadInt64(&t.ActiveOps) < atomic.LoadInt64(&best.ActiveOps) {
			best = t
		}
	}

	if best == nil {
		return nil, errors.New("no healthy targets available")
	}
	atomic.AddInt64(&best.ActiveOps, 1)
	return best, nil
}

func (lb *LoadBalancer) Release(targetID string) {
	lb.mu.RLock()
	defer lb.mu.RUnlock()
	for _, t := range lb.targets {
		if t.ID == targetID {
			atomic.AddInt64(&t.ActiveOps, -1)
			break
		}
	}
}
