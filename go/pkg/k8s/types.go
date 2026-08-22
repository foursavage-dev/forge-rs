package k8s

import "time"

type WorkerPoolSpec struct {
	Name            string            `json:"name"`
	MinReplicas     int               `json:"min_replicas"`
	MaxReplicas     int               `json:"max_replicas"`
	TargetCPULoad   int               `json:"target_cpu_load"`
	Toolchains      []string          `json:"toolchains"`
	NodeSelector    map[string]string `json:"node_selector"`
	ResourceLimitMB int64             `json:"resource_limit_mb"`
	SpotEnabled     bool              `json:"spot_enabled"`
	SpotMaxPrice    string            `json:"spot_max_price,omitempty"`
	Regions         []string          `json:"regions,omitempty"`
	Priority        int               `json:"priority"`
}

type WorkerPoolStatus struct {
	CurrentReplicas   int       `json:"current_replicas"`
	AvailableReplicas int       `json:"available_replicas"`
	ReadyReplicas     int       `json:"ready_replicas"`
	LastScaleTime     time.Time `json:"last_scale_time"`
	HealthStatus      string    `json:"health_status"`
	SpotReplicas      int       `json:"spot_replicas"`
	OnDemandReplicas  int       `json:"on_demand_replicas"`
	PreemptionsLastHour int     `json:"preemptions_last_hour"`
}

type FishClusterConfig struct {
	ClusterID        string           `json:"cluster_id"`
	Namespace        string           `json:"namespace"`
	CoordinatorAddr  string           `json:"coordinator_addr"`
	DefaultPool      WorkerPoolSpec   `json:"default_pool"`
	CustomPools      []WorkerPoolSpec `json:"custom_pools"`
	EnableSpot       bool             `json:"enable_spot"`
	EnableCrossRegion bool            `json:"enable_cross_region"`
	Regions          []string         `json:"regions"`
	ReplicationFactor int             `json:"replication_factor"`
	CacheEndpoint    string           `json:"cache_endpoint,omitempty"`
}

type CacheReplicationSpec struct {
	Enabled           bool     `json:"enabled"`
	Regions           []string `json:"regions"`
	ReplicationFactor int      `json:"replication_factor"`
	SyncIntervalSec   int      `json:"sync_interval_sec"`
}

type SpotInstanceSpec struct {
	Enabled            bool          `json:"enabled"`
	MaxPrice           string        `json:"max_price"`
	FallbackToOnDemand bool          `json:"fallback_to_on_demand"`
	PreemptionHandling string        `json:"preemption_handling"` // migrate, restart, queue
	GracePeriodSec     int           `json:"grace_period_sec"`
	CheckpointEnabled  bool          `json:"checkpoint_enabled"`
}

type AutoscalingPolicy struct {
	ScaleUpCooldownSec   int `json:"scale_up_cooldown_sec"`
	ScaleDownCooldownSec int `json:"scale_down_cooldown_sec"`
	TargetQueueDepth     int `json:"target_queue_depth"`
	MaxScaleUpStep       int `json:"max_scale_up_step"`
}
