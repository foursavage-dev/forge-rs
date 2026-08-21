package k8s

import (
	"sync"
	"time"
)

type SpotPreemptionNotice struct {
	WorkerID    string    `json:"worker_id"`
	GracePeriod time.Duration `json:"grace_period"`
	ReceivedAt  time.Time `json:"received_at"`
}

type SpotLifecycleManager struct {
	mu          sync.RWMutex
	inFlight    map[string][]string
	preemptions []SpotPreemptionNotice
}

func NewSpotLifecycleManager() *SpotLifecycleManager {
	return &SpotLifecycleManager{
		inFlight:    make(map[string][]string),
		preemptions: make([]SpotPreemptionNotice, 0),
	}
}

func (m *SpotLifecycleManager) RegisterTask(workerID string, taskID string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.inFlight[workerID] = append(m.inFlight[workerID], taskID)
}

func (m *SpotLifecycleManager) HandlePreemption(workerID string, grace time.Duration) []string {
	m.mu.Lock()
	defer m.mu.Unlock()

	notice := SpotPreemptionNotice{
		WorkerID:    workerID,
		GracePeriod: grace,
		ReceivedAt:  time.Now(),
	}
	m.preemptions = append(m.preemptions, notice)

	evacuateTasks := m.inFlight[workerID]
	delete(m.inFlight, workerID)
	return evacuateTasks
}
