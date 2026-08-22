package k8s

import (
	"context"
	"fmt"
	"sync"
	"time"
)

type ReconcileResult struct {
	Requeue      bool
	RequeueAfter time.Duration
	ScaledPools  map[string]int
}

type ClusterReconciler struct {
	mu          sync.RWMutex
	autoscalers map[string]*Autoscaler
}

func NewClusterReconciler() *ClusterReconciler {
	return &ClusterReconciler{
		autoscalers: make(map[string]*Autoscaler),
	}
}

func (r *ClusterReconciler) Reconcile(ctx context.Context, cluster *FishClusterConfig, queuedTasks int, avgTaskTime float64) (*ReconcileResult, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	if cluster == nil {
		return nil, fmt.Errorf("cluster config cannot be nil")
	}

	res := &ReconcileResult{
		ScaledPools: make(map[string]int),
	}

	scaler, exists := r.autoscalers[cluster.DefaultPool.Name]
	if !exists {
		scaler = NewAutoscaler(cluster.DefaultPool)
		r.autoscalers[cluster.DefaultPool.Name] = scaler
	}

	desired := scaler.CalculateDesiredReplicas(queuedTasks, avgTaskTime, 5.0)
	current, err := scaler.Scale(desired)
	if err != nil {
		return nil, err
	}
	res.ScaledPools[cluster.DefaultPool.Name] = current

	for _, pool := range cluster.CustomPools {
		poolScaler, pExists := r.autoscalers[pool.Name]
		if !pExists {
			poolScaler = NewAutoscaler(pool)
			r.autoscalers[pool.Name] = poolScaler
		}
		pDesired := poolScaler.CalculateDesiredReplicas(queuedTasks, avgTaskTime, 5.0)
		pCurrent, pErr := poolScaler.Scale(pDesired)
		if pErr == nil {
			res.ScaledPools[pool.Name] = pCurrent
		}
	}

	res.Requeue = true
	res.RequeueAfter = 10 * time.Second
	return res, nil
}

func (r *ClusterReconciler) GetPoolStatus(poolName string) (WorkerPoolStatus, bool) {
	r.mu.RLock()
	defer r.mu.RUnlock()

	scaler, exists := r.autoscalers[poolName]
	if !exists {
		return WorkerPoolStatus{}, false
	}
	return scaler.GetStatus(), true
}
