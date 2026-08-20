package gateway

import (
	"errors"
	"sync"
	"sync/atomic"
)

type WorkerTarget struct {
	ID        string `json:"id"`
	Address   string `json:"address"`
	Weight    int    `json:"weight"`
	ActiveOps int64  `json:"active_ops"`
	Healthy   bool   `json:"healthy"`
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
	lb.AddTargetWithWeight(id, address, 1)
}

func (lb *LoadBalancer) AddTargetWithWeight(id, address string, weight int) {
	lb.mu.Lock()
	defer lb.mu.Unlock()
	if weight <= 0 {
		weight = 1
	}
	for _, t := range lb.targets {
		if t.ID == id {
			t.Address = address
			t.Weight = weight
			t.Healthy = true
			return
		}
	}
	lb.targets = append(lb.targets, &WorkerTarget{
		ID:      id,
		Address: address,
		Weight:  weight,
		Healthy: true,
	})
}

func (lb *LoadBalancer) RemoveTarget(id string) {
	lb.mu.Lock()
	defer lb.mu.Unlock()
	for i, t := range lb.targets {
		if t.ID == id {
			lb.targets = append(lb.targets[:i], lb.targets[i+1:]...)
			return
		}
	}
}

func (lb *LoadBalancer) SetHealth(targetID string, healthy bool) {
	lb.mu.Lock()
	defer lb.mu.Unlock()
	for _, t := range lb.targets {
		if t.ID == targetID {
			t.Healthy = healthy
			return
		}
	}
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

func (lb *LoadBalancer) SelectRoundRobin() (*WorkerTarget, error) {
	lb.mu.RLock()
	defer lb.mu.RUnlock()

	var healthy []*WorkerTarget
	for _, t := range lb.targets {
		if t.Healthy {
			healthy = append(healthy, t)
		}
	}

	if len(healthy) == 0 {
		return nil, errors.New("no healthy targets available")
	}

	idx := atomic.AddUint64(&lb.counter, 1) % uint64(len(healthy))
	selected := healthy[idx]
	atomic.AddInt64(&selected.ActiveOps, 1)
	return selected, nil
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

func (lb *LoadBalancer) TargetCount() int {
	lb.mu.RLock()
	defer lb.mu.RUnlock()
	return len(lb.targets)
}
