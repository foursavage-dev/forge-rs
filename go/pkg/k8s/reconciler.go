package k8s

import (
	"context"
	"fmt"
	"sync"
	"time"
)

type ReconcileResult struct {
	Requeue           bool
	RequeueAfter      time.Duration
	ScaledPools       map[string]int
	SpotMigrations    []MigrationResult
	CacheSyncStatus   *CacheSyncStatus
	Message           string
}

type ClusterReconciler struct {
	mu                sync.RWMutex
	autoscalers       map[string]*Autoscaler
	spotManager       *SpotLifecycleManager
	cacheReplicator   *CacheReplicationTracker
	lastReconcileTime time.Time
	reconcileCount    int
}

type CacheReplicationTracker struct {
	mu               sync.RWMutex
	regions          []string
	pendingArtifacts int
	syncedArtifacts  int
	lastSyncTime     time.Time
	syncRate         float64
}

func NewCacheReplicationTracker(regions []string) *CacheReplicationTracker {
	return &CacheReplicationTracker{
		regions:          regions,
		pendingArtifacts: 0,
		syncedArtifacts:  0,
		lastSyncTime:     time.Now(),
		syncRate:         0.0,
	}
}

func (c *CacheReplicationTracker) RecordSync(success bool, count int) {
	c.mu.Lock()
	defer c.mu.Unlock()
	
	if success {
		c.syncedArtifacts += count
		c.pendingArtifacts -= count
		if c.pendingArtifacts < 0 {
			c.pendingArtifacts = 0
		}
	} else {
		c.pendingArtifacts += count
	}
	c.lastSyncTime = time.Now()
	
	total := c.syncedArtifacts + c.pendingArtifacts
	if total > 0 {
		c.syncRate = float64(c.syncedArtifacts) / float64(total)
	}
}

func (c *CacheReplicationTracker) GetStatus() CacheSyncStatus {
	c.mu.RLock()
	defer c.mu.RUnlock()
	
	return CacheSyncStatus{
		RegionsSynced:    len(c.regions),
		PendingArtifacts: c.pendingArtifacts,
		LastSyncTime:     c.lastSyncTime.Format(time.RFC3339),
		SyncRate:         c.syncRate,
	}
}

func NewClusterReconciler() *ClusterReconciler {
	return &ClusterReconciler{
		autoscalers:       make(map[string]*Autoscaler),
		spotManager:       NewSpotLifecycleManager(),
		cacheReplicator:   NewCacheReplicationTracker([]string{"us-east-1", "us-west-2", "eu-west-1"}),
		lastReconcileTime: time.Now(),
		reconcileCount:    0,
	}
}

func NewClusterReconcilerWithSpotManager(spotManager *SpotLifecycleManager) *ClusterReconciler {
	return &ClusterReconciler{
		autoscalers:       make(map[string]*Autoscaler),
		spotManager:       spotManager,
		cacheReplicator:   NewCacheReplicationTracker([]string{"us-east-1", "us-west-2", "eu-west-1"}),
		lastReconcileTime: time.Now(),
		reconcileCount:    0,
	}
}

func (r *ClusterReconciler) Reconcile(ctx context.Context, cluster *FishClusterConfig, queuedTasks int, avgTaskTime float64) (*ReconcileResult, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	if cluster == nil {
		return nil, fmt.Errorf("cluster config cannot be nil")
	}

	r.reconcileCount++
	r.lastReconcileTime = time.Now()

	res := &ReconcileResult{
		ScaledPools: make(map[string]int),
	}

	// Handle spot instance preemptions if enabled
	if cluster.EnableSpot {
		// Check for preemptions and migrate tasks
		preemptions := r.spotManager.GetPreemptionHistory()
		if len(preemptions) > 0 {
			lastPreemption := preemptions[len(preemptions)-1]
			if time.Since(lastPreemption.ReceivedAt) < 5*time.Minute {
				// Recent preemption, handle migration
				availableWorkers := r.getAvailableWorkers(cluster)
				migrations := r.spotManager.MigrateTasks(ctx, []string{"task-recovery"}, lastPreemption.WorkerID, availableWorkers)
				res.SpotMigrations = migrations
			}
		}
	}

	// Reconcile default pool
	scaler, exists := r.autoscalers[cluster.DefaultPool.Name]
	if !exists {
		scaler = NewAutoscaler(cluster.DefaultPool)
		r.autoscalers[cluster.DefaultPool.Name] = scaler
	}

	// Record metrics for predictive scaling
	scaler.RecordMetrics(queuedTasks, avgTaskTime, 75.0) // Assume 75% CPU

	desired := scaler.CalculateDesiredReplicas(queuedTasks, avgTaskTime, 5.0)
	
	// Consider predictive scaling for next hour
	predicted := scaler.PredictNextHourDemand()
	if predicted > desired {
		// If we predict higher demand, scale up proactively (but not too aggressively)
		desired = (desired + predicted) / 2
	}

	current, err := scaler.Scale(desired)
	if err != nil {
		// If cooldown active, keep current
		if err.Error() == "scale-up cooldown active" || err.Error() == "scale-down cooldown active" {
			current = scaler.GetStatus().CurrentReplicas
		} else {
			return nil, err
		}
	}
	res.ScaledPools[cluster.DefaultPool.Name] = current

	// Reconcile custom pools
	for _, pool := range cluster.CustomPools {
		poolScaler, pExists := r.autoscalers[pool.Name]
		if !pExists {
			poolScaler = NewAutoscaler(pool)
			r.autoscalers[pool.Name] = poolScaler
		}
		
		poolScaler.RecordMetrics(queuedTasks, avgTaskTime, 75.0)
		pDesired := poolScaler.CalculateDesiredReplicas(queuedTasks, avgTaskTime, 5.0)
		pCurrent, pErr := poolScaler.Scale(pDesired)
		if pErr == nil {
			res.ScaledPools[pool.Name] = pCurrent
		} else if pErr.Error() == "scale-up cooldown active" || pErr.Error() == "scale-down cooldown active" {
			res.ScaledPools[pool.Name] = poolScaler.GetStatus().CurrentReplicas
		}
	}

	// Handle cross-region cache replication
	if cluster.EnableCrossRegion {
		cacheStatus := r.cacheReplicator.GetStatus()
		res.CacheSyncStatus = &cacheStatus
		
		// Simulate cache replication
		if queuedTasks > 0 {
			r.cacheReplicator.RecordSync(true, queuedTasks/10)
		}
	}

	res.Requeue = true
	res.RequeueAfter = 10 * time.Second
	res.Message = fmt.Sprintf("Reconciled cluster %s: %d pools scaled, %d spot migrations",
		cluster.ClusterID, len(res.ScaledPools), len(res.SpotMigrations))

	return res, nil
}

func (r *ClusterReconciler) getAvailableWorkers(cluster *FishClusterConfig) []string {
	var workers []string
	
	// Collect workers from all pools
	for _, pool := range append([]WorkerPoolSpec{cluster.DefaultPool}, cluster.CustomPools...) {
		if scaler, exists := r.autoscalers[pool.Name]; exists {
			status := scaler.GetStatus()
			for i := 0; i < status.AvailableReplicas; i++ {
				workers = append(workers, fmt.Sprintf("%s-worker-%d", pool.Name, i))
			}
		}
	}
	
	return workers
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

func (r *ClusterReconciler) GetAllPoolStatuses() map[string]WorkerPoolStatus {
	r.mu.RLock()
	defer r.mu.RUnlock()

	statuses := make(map[string]WorkerPoolStatus)
	for name, scaler := range r.autoscalers {
		statuses[name] = scaler.GetStatus()
	}
	return statuses
}

func (r *ClusterReconciler) GetSpotManager() *SpotLifecycleManager {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return r.spotManager
}

func (r *ClusterReconciler) GetReconcileCount() int {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return r.reconcileCount
}

func (r *ClusterReconciler) HandleSpotPreemption(notice SpotPreemptionNotice) []string {
	return r.spotManager.HandlePreemptionWithDetails(notice)
}
