package coordinator

import (
	"errors"
	"sync"
	"time"
)

type WorkerNode struct {
	ID                  string            `json:"id"`
	Address             string            `json:"address"`
	CPUCores            int               `json:"cpu_cores"`
	MemoryBytes         int64             `json:"memory_bytes"`
	SupportedToolchains []string          `json:"supported_toolchains"`
	Tags                map[string]string `json:"tags"`
	ActiveJobs          int               `json:"active_jobs"`
	LastHeartbeat       time.Time         `json:"last_heartbeat"`
}

type NodeRegistry struct {
	mu      sync.RWMutex
	workers map[string]*WorkerNode
	timeout time.Duration
}

func NewNodeRegistry(heartbeatTimeout time.Duration) *NodeRegistry {
	return &NodeRegistry{
		workers: make(map[string]*WorkerNode),
		timeout: heartbeatTimeout,
	}
}

func (r *NodeRegistry) Register(node *WorkerNode) error {
	if node.ID == "" || node.Address == "" {
		return errors.New("worker id and address must not be empty")
	}
	r.mu.Lock()
	defer r.mu.Unlock()
	node.LastHeartbeat = time.Now()
	r.workers[node.ID] = node
	return nil
}

func (r *NodeRegistry) Heartbeat(workerID string, activeJobs int) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	node, exists := r.workers[workerID]
	if !exists {
		return errors.New("worker not registered")
	}
	node.ActiveJobs = activeJobs
	node.LastHeartbeat = time.Now()
	return nil
}

func (r *NodeRegistry) ListHealthyWorkers() []*WorkerNode {
	r.mu.RLock()
	defer r.mu.RUnlock()
	cutoff := time.Now().Add(-r.timeout)
	var healthy []*WorkerNode
	for _, w := range r.workers {
		if w.LastHeartbeat.After(cutoff) {
			healthy = append(healthy, w)
		}
	}
	return healthy
}

func (r *NodeRegistry) SelectOptimalWorker(toolchain string) (*WorkerNode, error) {
	healthy := r.ListHealthyWorkers()
	if len(healthy) == 0 {
		return nil, errors.New("no healthy workers available")
	}

	var best *WorkerNode
	for _, w := range healthy {
		supports := false
		for _, tc := range w.SupportedToolchains {
			if tc == toolchain || tc == "*" {
				supports = true
				break
			}
		}
		if !supports {
			continue
		}
		if best == nil || w.ActiveJobs < best.ActiveJobs {
			best = w
		}
	}

	if best == nil {
		return nil, errors.New("no suitable worker supporting toolchain: " + toolchain)
	}
	return best, nil
}
