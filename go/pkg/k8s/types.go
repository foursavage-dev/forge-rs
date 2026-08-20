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
}

type WorkerPoolStatus struct {
	CurrentReplicas   int       `json:"current_replicas"`
	AvailableReplicas int       `json:"available_replicas"`
	LastScaleTime     time.Time `json:"last_scale_time"`
	HealthStatus      string    `json:"health_status"`
}

type FishClusterConfig struct {
	ClusterID        string           `json:"cluster_id"`
	Namespace        string           `json:"namespace"`
	CoordinatorAddr  string           `json:"coordinator_addr"`
	DefaultPool      WorkerPoolSpec   `json:"default_pool"`
	CustomPools      []WorkerPoolSpec `json:"custom_pools"`
}
