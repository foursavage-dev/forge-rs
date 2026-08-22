package k8s

import (
	"errors"
	"math"
	"sync"
	"time"
)

type Autoscaler struct {
	mu     sync.RWMutex
	spec   WorkerPoolSpec
	status WorkerPoolStatus
}

func NewAutoscaler(spec WorkerPoolSpec) *Autoscaler {
	return &Autoscaler{
		spec: spec,
		status: WorkerPoolStatus{
			CurrentReplicas:   spec.MinReplicas,
			AvailableReplicas: spec.MinReplicas,
			LastScaleTime:     time.Now(),
			HealthStatus:      "Healthy",
		},
	}
}

func (a *Autoscaler) CalculateDesiredReplicas(queuedTasks int, avgTaskTimeSec float64, targetWaitSec float64) int {
	a.mu.RLock()
	defer a.mu.RUnlock()

	if targetWaitSec <= 0 {
		targetWaitSec = 10.0
	}
	requiredThroughput := float64(queuedTasks) / targetWaitSec
	neededWorkers := int(math.Ceil(requiredThroughput * avgTaskTimeSec))

	if neededWorkers < a.spec.MinReplicas {
		return a.spec.MinReplicas
	}
	if neededWorkers > a.spec.MaxReplicas {
		return a.spec.MaxReplicas
	}
	return neededWorkers
}

func (a *Autoscaler) Scale(desired int) (int, error) {
	a.mu.Lock()
	defer a.mu.Unlock()

	if desired < a.spec.MinReplicas || desired > a.spec.MaxReplicas {
		return a.status.CurrentReplicas, errors.New("desired replicas out of bounds")
	}

	a.status.CurrentReplicas = desired
	a.status.AvailableReplicas = desired
	a.status.LastScaleTime = time.Now()
	return a.status.CurrentReplicas, nil
}

func (a *Autoscaler) GetStatus() WorkerPoolStatus {
	a.mu.RLock()
	defer a.mu.RUnlock()
	return a.status
}
