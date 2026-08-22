package k8s

import (
	"context"
	"fmt"
	"sync"
	"time"
)

type SpotPreemptionNotice struct {
	WorkerID    string        `json:"worker_id"`
	GracePeriod time.Duration `json:"grace_period"`
	ReceivedAt  time.Time     `json:"received_at"`
	InstanceType string       `json:"instance_type"`
	Region      string        `json:"region"`
	Reason      string        `json:"reason"`
}

type TaskCheckpoint struct {
	TaskID      string    `json:"task_id"`
	WorkerID    string    `json:"worker_id"`
	CheckpointAt time.Time `json:"checkpoint_at"`
	Progress    float64   `json:"progress"`
	Data        []byte    `json:"data,omitempty"`
}

type MigrationResult struct {
	TaskID         string        `json:"task_id"`
	FromWorker     string        `json:"from_worker"`
	ToWorker       string        `json:"to_worker"`
	Success        bool          `json:"success"`
	Duration       time.Duration `json:"duration"`
	CheckpointUsed bool          `json:"checkpoint_used"`
	Error          string        `json:"error,omitempty"`
}

type SpotLifecycleManager struct {
	mu              sync.RWMutex
	inFlight        map[string][]string // workerID -> taskIDs
	checkpoints     map[string]TaskCheckpoint
	preemptions     []SpotPreemptionNotice
	migrationHistory []MigrationResult
	spotWorkers     map[string]bool
	onDemandPool    []string
	maxRetries      int
}

func NewSpotLifecycleManager() *SpotLifecycleManager {
	return &SpotLifecycleManager{
		inFlight:         make(map[string][]string),
		checkpoints:      make(map[string]TaskCheckpoint),
		preemptions:      make([]SpotPreemptionNotice, 0),
		migrationHistory: make([]MigrationResult, 0),
		spotWorkers:      make(map[string]bool),
		onDemandPool:     make([]string, 0),
		maxRetries:       3,
	}
}

func (m *SpotLifecycleManager) RegisterWorker(workerID string, isSpot bool) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.spotWorkers[workerID] = isSpot
	if !isSpot {
		m.onDemandPool = append(m.onDemandPool, workerID)
	}
}

func (m *SpotLifecycleManager) RegisterTask(workerID string, taskID string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.inFlight[workerID] = append(m.inFlight[workerID], taskID)
}

func (m *SpotLifecycleManager) CheckpointTask(checkpoint TaskCheckpoint) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.checkpoints[checkpoint.TaskID] = checkpoint
}

func (m *SpotLifecycleManager) GetCheckpoint(taskID string) (TaskCheckpoint, bool) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	cp, ok := m.checkpoints[taskID]
	return cp, ok
}

func (m *SpotLifecycleManager) HandlePreemption(workerID string, grace time.Duration) []string {
	m.mu.Lock()
	defer m.mu.Unlock()

	notice := SpotPreemptionNotice{
		WorkerID:    workerID,
		GracePeriod: grace,
		ReceivedAt:  time.Now(),
		Reason:      "Spot interruption notice",
	}
	m.preemptions = append(m.preemptions, notice)

	evacuateTasks := m.inFlight[workerID]
	delete(m.inFlight, workerID)
	return evacuateTasks
}

func (m *SpotLifecycleManager) HandlePreemptionWithDetails(notice SpotPreemptionNotice) []string {
	m.mu.Lock()
	defer m.mu.Unlock()

	if notice.ReceivedAt.IsZero() {
		notice.ReceivedAt = time.Now()
	}
	if notice.GracePeriod == 0 {
		notice.GracePeriod = 2 * time.Minute
	}

	m.preemptions = append(m.preemptions, notice)
	evacuateTasks := m.inFlight[notice.WorkerID]
	delete(m.inFlight, notice.WorkerID)
	return evacuateTasks
}

// Fault-tolerant task migration upon cloud node preemption
func (m *SpotLifecycleManager) MigrateTasks(
	ctx context.Context,
	tasks []string,
	fromWorker string,
	availableWorkers []string,
) []MigrationResult {
	m.mu.Lock()
	defer m.mu.Unlock()

	var results []MigrationResult

	if len(availableWorkers) == 0 {
		// No workers available, queue tasks for later
		for _, taskID := range tasks {
			results = append(results, MigrationResult{
				TaskID:     taskID,
				FromWorker: fromWorker,
				Success:    false,
				Error:      "no available workers for migration",
			})
		}
		m.migrationHistory = append(m.migrationHistory, results...)
		return results
	}

	workerIdx := 0
	for _, taskID := range tasks {
		start := time.Now()
		
		// Check if checkpoint exists
		checkpoint, hasCheckpoint := m.checkpoints[taskID]
		
		// Select next available worker (round-robin)
		toWorker := availableWorkers[workerIdx%len(availableWorkers)]
		workerIdx++

		// Simulate migration
		result := MigrationResult{
			TaskID:         taskID,
			FromWorker:     fromWorker,
			ToWorker:       toWorker,
			Success:        true,
			Duration:       time.Since(start),
			CheckpointUsed: hasCheckpoint,
		}

		if hasCheckpoint {
			// If we have checkpoint, we can resume from progress
			result.Duration += time.Duration(float64(time.Second) * (1.0 - checkpoint.Progress))
		}

		// Add to in-flight for new worker
		m.inFlight[toWorker] = append(m.inFlight[toWorker], taskID)
		results = append(results, result)
	}

	m.migrationHistory = append(m.migrationHistory, results...)
	return results
}

func (m *SpotLifecycleManager) GetPreemptionHistory() []SpotPreemptionNotice {
	m.mu.RLock()
	defer m.mu.RUnlock()
	copied := make([]SpotPreemptionNotice, len(m.preemptions))
	copy(copied, m.preemptions)
	return copied
}

func (m *SpotLifecycleManager) GetMigrationHistory() []MigrationResult {
	m.mu.RLock()
	defer m.mu.RUnlock()
	copied := make([]MigrationResult, len(m.migrationHistory))
	copy(copied, m.migrationHistory)
	return copied
}

func (m *SpotLifecycleManager) GetInFlightTasks(workerID string) []string {
	m.mu.RLock()
	defer m.mu.RUnlock()
	tasks := m.inFlight[workerID]
	copied := make([]string, len(tasks))
	copy(copied, tasks)
	return copied
}

func (m *SpotLifecycleManager) IsSpotWorker(workerID string) bool {
	m.mu.RLock()
	defer m.mu.RUnlock()
	isSpot, exists := m.spotWorkers[workerID]
	return exists && isSpot
}

func (m *SpotLifecycleManager) GetSpotWorkerCount() int {
	m.mu.RLock()
	defer m.mu.RUnlock()
	count := 0
	for _, isSpot := range m.spotWorkers {
		if isSpot {
			count++
		}
	}
	return count
}

func (m *SpotLifecycleManager) GetOnDemandWorkerCount() int {
	m.mu.RLock()
	defer m.mu.RUnlock()
	return len(m.onDemandPool)
}

func (m *SpotLifecycleManager) CalculatePreemptionRateLastHour() float64 {
	m.mu.RLock()
	defer m.mu.RUnlock()
	
	cutoff := time.Now().Add(-1 * time.Hour)
	count := 0
	for _, notice := range m.preemptions {
		if notice.ReceivedAt.After(cutoff) {
			count++
		}
	}
	return float64(count)
}

func (m *SpotLifecycleManager) ShouldUseSpotForTask(taskPriority int, estimatedDuration time.Duration) bool {
	// Use spot for low-priority, short tasks to save cost
	// Use on-demand for high-priority, long tasks for reliability
	
	if taskPriority >= 8 {
		// High priority tasks should use on-demand
		return false
	}
	
	if estimatedDuration > 30*time.Minute {
		// Long tasks have higher preemption risk
		preemptionRate := m.CalculatePreemptionRateLastHour()
		if preemptionRate > 5 {
			return false
		}
	}
	
	return true
}

func (m *SpotLifecycleManager) GenerateSpotRecommendation() string {
	spotCount := m.GetSpotWorkerCount()
	onDemandCount := m.GetOnDemandWorkerCount()
	preemptionRate := m.CalculatePreemptionRateLastHour()
	
	total := spotCount + onDemandCount
	if total == 0 {
		return "No workers registered"
	}
	
	spotRatio := float64(spotCount) / float64(total)
	
	if preemptionRate > 10 {
		return fmt.Sprintf("High preemption rate (%.1f/hour), recommend reducing spot ratio from %.0f%% to 50%%", preemptionRate, spotRatio*100)
	}
	
	if spotRatio < 0.7 && preemptionRate < 3 {
		return fmt.Sprintf("Low preemption rate (%.1f/hour), can increase spot ratio from %.0f%% to 80%% for cost savings", preemptionRate, spotRatio*100)
	}
	
	return fmt.Sprintf("Spot ratio %.0f%% is optimal (preemption rate: %.1f/hour)", spotRatio*100, preemptionRate)
}
