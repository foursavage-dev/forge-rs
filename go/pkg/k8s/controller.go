package k8s

import (
	"errors"
	"math"
	"sync"
	"time"
)

type Autoscaler struct {
	mu                 sync.RWMutex
	spec               WorkerPoolSpec
	status             WorkerPoolStatus
	policy             AutoscalingPolicy
	lastScaleUpTime    time.Time
	lastScaleDownTime  time.Time
	metricsHistory     []ScalingMetric
}

type ScalingMetric struct {
	Timestamp      time.Time
	QueuedTasks    int
	AvgTaskTimeSec float64
	ActiveWorkers  int
	CPUUsage       float64
}

func NewAutoscaler(spec WorkerPoolSpec) *Autoscaler {
	return &Autoscaler{
		spec: spec,
		status: WorkerPoolStatus{
			CurrentReplicas:   spec.MinReplicas,
			AvailableReplicas: spec.MinReplicas,
			ReadyReplicas:     spec.MinReplicas,
			LastScaleTime:     time.Now(),
			HealthStatus:      "Healthy",
		},
		policy: AutoscalingPolicy{
			ScaleUpCooldownSec:   60,
			ScaleDownCooldownSec: 300,
			TargetQueueDepth:     10,
			MaxScaleUpStep:       5,
		},
		metricsHistory: make([]ScalingMetric, 0),
	}
}

func NewAutoscalerWithPolicy(spec WorkerPoolSpec, policy AutoscalingPolicy) *Autoscaler {
	scaler := NewAutoscaler(spec)
	scaler.policy = policy
	return scaler
}

func (a *Autoscaler) CalculateDesiredReplicas(queuedTasks int, avgTaskTimeSec float64, targetWaitSec float64) int {
	a.mu.RLock()
	defer a.mu.RUnlock()

	if targetWaitSec <= 0 {
		targetWaitSec = 10.0
	}

	// Little's Law: L = λ * W
	// Required throughput = queuedTasks / targetWaitSec
	// Workers needed = throughput * avgTaskTime
	requiredThroughput := float64(queuedTasks) / targetWaitSec
	neededWorkers := int(math.Ceil(requiredThroughput * avgTaskTimeSec))

	// Consider current metrics history for smoothing
	if len(a.metricsHistory) > 0 {
		// Use exponential moving average to avoid flapping
		recentAvg := a.calculateRecentAverageQueueDepth()
		if recentAvg > 0 {
			smoothedQueue := int(float64(queuedTasks)*0.7 + recentAvg*0.3)
			smoothedThroughput := float64(smoothedQueue) / targetWaitSec
			smoothedWorkers := int(math.Ceil(smoothedThroughput * avgTaskTimeSec))
			// Use max to be conservative on scale-up, min on scale-down
			if smoothedWorkers > neededWorkers {
				neededWorkers = smoothedWorkers
			}
		}
	}

	if neededWorkers < a.spec.MinReplicas {
		return a.spec.MinReplicas
	}
	if neededWorkers > a.spec.MaxReplicas {
		return a.spec.MaxReplicas
	}
	return neededWorkers
}

func (a *Autoscaler) calculateRecentAverageQueueDepth() float64 {
	if len(a.metricsHistory) == 0 {
		return 0
	}
	
	// Last 5 metrics
	count := 5
	if len(a.metricsHistory) < count {
		count = len(a.metricsHistory)
	}
	
	sum := 0
	for i := len(a.metricsHistory) - count; i < len(a.metricsHistory); i++ {
		sum += a.metricsHistory[i].QueuedTasks
	}
	return float64(sum) / float64(count)
}

func (a *Autoscaler) Scale(desired int) (int, error) {
	a.mu.Lock()
	defer a.mu.Unlock()

	if desired < a.spec.MinReplicas || desired > a.spec.MaxReplicas {
		return a.status.CurrentReplicas, errors.New("desired replicas out of bounds")
	}

	// Check cooldown periods
	now := time.Now()
	if desired > a.status.CurrentReplicas {
		// Scale up - check cooldown
		if now.Sub(a.lastScaleUpTime).Seconds() < float64(a.policy.ScaleUpCooldownSec) {
			return a.status.CurrentReplicas, errors.New("scale-up cooldown active")
		}
		
		// Limit scale-up step
		maxAllowed := a.status.CurrentReplicas + a.policy.MaxScaleUpStep
		if desired > maxAllowed {
			desired = maxAllowed
		}
		
		a.lastScaleUpTime = now
	} else if desired < a.status.CurrentReplicas {
		// Scale down - check cooldown
		if now.Sub(a.lastScaleDownTime).Seconds() < float64(a.policy.ScaleDownCooldownSec) {
			return a.status.CurrentReplicas, errors.New("scale-down cooldown active")
		}
		a.lastScaleDownTime = now
	}

	a.status.CurrentReplicas = desired
	a.status.AvailableReplicas = desired
	a.status.ReadyReplicas = desired // Assume immediate readiness for simulation
	a.status.LastScaleTime = now
	
	// Update spot vs on-demand breakdown
	if a.spec.SpotEnabled {
		// 70% spot, 30% on-demand for cost optimization
		spotCount := int(float64(desired) * 0.7)
		a.status.SpotReplicas = spotCount
		a.status.OnDemandReplicas = desired - spotCount
	} else {
		a.status.SpotReplicas = 0
		a.status.OnDemandReplicas = desired
	}

	return a.status.CurrentReplicas, nil
}

func (a *Autoscaler) GetStatus() WorkerPoolStatus {
	a.mu.RLock()
	defer a.mu.RUnlock()
	return a.status
}

func (a *Autoscaler) RecordMetrics(queuedTasks int, avgTaskTimeSec float64, cpuUsage float64) {
	a.mu.Lock()
	defer a.mu.Unlock()

	metric := ScalingMetric{
		Timestamp:      time.Now(),
		QueuedTasks:    queuedTasks,
		AvgTaskTimeSec: avgTaskTimeSec,
		ActiveWorkers:  a.status.CurrentReplicas,
		CPUUsage:       cpuUsage,
	}

	a.metricsHistory = append(a.metricsHistory, metric)
	
	// Keep only last 100 metrics
	if len(a.metricsHistory) > 100 {
		a.metricsHistory = a.metricsHistory[len(a.metricsHistory)-100:]
	}
}

func (a *Autoscaler) GetMetricsHistory() []ScalingMetric {
	a.mu.RLock()
	defer a.mu.RUnlock()
	copied := make([]ScalingMetric, len(a.metricsHistory))
	copy(copied, a.metricsHistory)
	return copied
}

func (a *Autoscaler) SetPolicy(policy AutoscalingPolicy) {
	a.mu.Lock()
	defer a.mu.Unlock()
	a.policy = policy
}

func (a *Autoscaler) GetPolicy() AutoscalingPolicy {
	a.mu.RLock()
	defer a.mu.RUnlock()
	return a.policy
}

// Predictive scaling based on historical patterns
func (a *Autoscaler) PredictNextHourDemand() int {
	a.mu.RLock()
	defer a.mu.RUnlock()

	if len(a.metricsHistory) < 10 {
		return a.status.CurrentReplicas
	}

	// Simple linear regression on queue depth
	// In production, would use more sophisticated ML model
	recent := a.metricsHistory[len(a.metricsHistory)-10:]
	
	var sumX, sumY, sumXY, sumX2 float64
	for i, m := range recent {
		x := float64(i)
		y := float64(m.QueuedTasks)
		sumX += x
		sumY += y
		sumXY += x * y
		sumX2 += x * x
	}
	
	n := float64(len(recent))
	denominator := n*sumX2 - sumX*sumX
	if denominator == 0 {
		return a.status.CurrentReplicas
	}
	
	slope := (n*sumXY - sumX*sumY) / denominator
	
	// Predict next hour (assuming 5min intervals, 12 steps)
	predictedQueue := recent[len(recent)-1].QueuedTasks + int(slope*12)
	if predictedQueue < 0 {
		predictedQueue = 0
	}
	
	// Calculate workers needed for predicted queue
	avgTaskTime := recent[len(recent)-1].AvgTaskTimeSec
	if avgTaskTime == 0 {
		avgTaskTime = 2.0
	}
	
	predictedWorkers := int(math.Ceil(float64(predictedQueue) / 10.0 * avgTaskTime))
	if predictedWorkers < a.spec.MinReplicas {
		predictedWorkers = a.spec.MinReplicas
	}
	if predictedWorkers > a.spec.MaxReplicas {
		predictedWorkers = a.spec.MaxReplicas
	}
	
	return predictedWorkers
}
