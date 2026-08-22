package k8s

import (
	"fmt"
	"strings"
)

type CRDMeta struct {
	APIVersion string `json:"apiVersion"`
	Kind       string `json:"kind"`
}

type FishClusterCRD struct {
	CRDMeta
	Metadata map[string]interface{} `json:"metadata"`
	Spec     FishClusterConfig      `json:"spec"`
	Status   ClusterStatus          `json:"status"`
}

type ClusterStatus struct {
	Phase           string             `json:"phase"`
	ActiveWorkers   int                `json:"active_workers"`
	PoolStatuses    []WorkerPoolStatus `json:"pool_statuses"`
	Message         string             `json:"message"`
	LastSyncTime    string             `json:"last_sync_time"`
	Conditions      []ClusterCondition `json:"conditions"`
	CacheSyncStatus *CacheSyncStatus   `json:"cache_sync_status,omitempty"`
	SpotStatus      *SpotStatus        `json:"spot_status,omitempty"`
}

type ClusterCondition struct {
	Type               string `json:"type"`
	Status             string `json:"status"`
	LastTransitionTime string `json:"lastTransitionTime"`
	Reason             string `json:"reason"`
	Message            string `json:"message"`
}

type CacheSyncStatus struct {
	RegionsSynced   int    `json:"regions_synced"`
	PendingArtifacts int   `json:"pending_artifacts"`
	LastSyncTime    string `json:"last_sync_time"`
	SyncRate        float64 `json:"sync_rate"`
}

type SpotStatus struct {
	TotalSpotReplicas    int `json:"total_spot_replicas"`
	PreemptionsLastHour  int `json:"preemptions_last_hour"`
	MigrationsSucceeded  int `json:"migrations_succeeded"`
	MigrationsFailed     int `json:"migrations_failed"`
}

func GenerateCRDManifestYAML() string {
	var b strings.Builder
	b.WriteString("apiVersion: apiextensions.k8s.io/v1\n")
	b.WriteString("kind: CustomResourceDefinition\n")
	b.WriteString("metadata:\n")
	b.WriteString("  name: fishclusters.fish.build\n")
	b.WriteString("  labels:\n")
	b.WriteString("    app.kubernetes.io/name: fish-operator\n")
	b.WriteString("    app.kubernetes.io/version: v0.4.0\n")
	b.WriteString("spec:\n")
	b.WriteString("  group: fish.build\n")
	b.WriteString("  names:\n")
	b.WriteString("    kind: FishCluster\n")
	b.WriteString("    listKind: FishClusterList\n")
	b.WriteString("    plural: fishclusters\n")
	b.WriteString("    singular: fishcluster\n")
	b.WriteString("    shortNames:\n")
	b.WriteString("    - fc\n")
	b.WriteString("  scope: Namespaced\n")
	b.WriteString("  versions:\n")
	b.WriteString("    - name: v1alpha1\n")
	b.WriteString("      served: true\n")
	b.WriteString("      storage: true\n")
	b.WriteString("      subresources:\n")
	b.WriteString("        status: {}\n")
	b.WriteString("        scale:\n")
	b.WriteString("          specReplicasPath: .spec.default_pool.max_replicas\n")
	b.WriteString("          statusReplicasPath: .status.active_workers\n")
	b.WriteString("      additionalPrinterColumns:\n")
	b.WriteString("      - name: ClusterID\n")
	b.WriteString("        type: string\n")
	b.WriteString("        jsonPath: .spec.cluster_id\n")
	b.WriteString("      - name: Workers\n")
	b.WriteString("        type: integer\n")
	b.WriteString("        jsonPath: .status.active_workers\n")
	b.WriteString("      - name: Phase\n")
	b.WriteString("        type: string\n")
	b.WriteString("        jsonPath: .status.phase\n")
	b.WriteString("      schema:\n")
	b.WriteString("        openAPIV3Schema:\n")
	b.WriteString("          type: object\n")
	b.WriteString("          required: [spec]\n")
	b.WriteString("          properties:\n")
	b.WriteString("            spec:\n")
	b.WriteString("              type: object\n")
	b.WriteString("              required: [cluster_id, default_pool]\n")
	b.WriteString("              properties:\n")
	b.WriteString("                cluster_id:\n")
	b.WriteString("                  type: string\n")
	b.WriteString("                  minLength: 1\n")
	b.WriteString("                namespace:\n")
	b.WriteString("                  type: string\n")
	b.WriteString("                coordinator_addr:\n")
	b.WriteString("                  type: string\n")
	b.WriteString("                enable_spot:\n")
	b.WriteString("                  type: boolean\n")
	b.WriteString("                enable_cross_region:\n")
	b.WriteString("                  type: boolean\n")
	b.WriteString("                replication_factor:\n")
	b.WriteString("                  type: integer\n")
	b.WriteString("                  minimum: 1\n")
	b.WriteString("                  maximum: 5\n")
	b.WriteString("                default_pool:\n")
	b.WriteString("                  type: object\n")
	b.WriteString("                  properties:\n")
	b.WriteString("                    name:\n")
	b.WriteString("                      type: string\n")
	b.WriteString("                    min_replicas:\n")
	b.WriteString("                      type: integer\n")
	b.WriteString("                      minimum: 0\n")
	b.WriteString("                    max_replicas:\n")
	b.WriteString("                      type: integer\n")
	b.WriteString("                      minimum: 1\n")
	b.WriteString("                    spot_enabled:\n")
	b.WriteString("                      type: boolean\n")
	b.WriteString("                    toolchains:\n")
	b.WriteString("                      type: array\n")
	b.WriteString("                      items:\n")
	b.WriteString("                        type: string\n")
	b.WriteString("            status:\n")
	b.WriteString("              type: object\n")
	b.WriteString("              properties:\n")
	b.WriteString("                phase:\n")
	b.WriteString("                  type: string\n")
	b.WriteString("                active_workers:\n")
	b.WriteString("                  type: integer\n")
	b.WriteString("                message:\n")
	b.WriteString("                  type: string\n")
	return b.String()
}

func GenerateClusterDeploymentYAML(config FishClusterConfig) string {
	var b strings.Builder
	b.WriteString("apiVersion: fish.build/v1alpha1\n")
	b.WriteString("kind: FishCluster\n")
	b.WriteString("metadata:\n")
	b.WriteString(fmt.Sprintf("  name: %s\n", config.ClusterID))
	b.WriteString(fmt.Sprintf("  namespace: %s\n", config.Namespace))
	b.WriteString("  labels:\n")
	b.WriteString("    app: fish-cluster\n")
	b.WriteString(fmt.Sprintf("    cluster: %s\n", config.ClusterID))
	b.WriteString("spec:\n")
	b.WriteString(fmt.Sprintf("  cluster_id: %s\n", config.ClusterID))
	b.WriteString(fmt.Sprintf("  coordinator_addr: %s\n", config.CoordinatorAddr))
	b.WriteString(fmt.Sprintf("  enable_spot: %t\n", config.EnableSpot))
	b.WriteString(fmt.Sprintf("  enable_cross_region: %t\n", config.EnableCrossRegion))
	b.WriteString(fmt.Sprintf("  replication_factor: %d\n", config.ReplicationFactor))
	if len(config.Regions) > 0 {
		b.WriteString("  regions:\n")
		for _, r := range config.Regions {
			b.WriteString(fmt.Sprintf("    - %s\n", r))
		}
	}
	b.WriteString("  default_pool:\n")
	b.WriteString(fmt.Sprintf("    name: %s\n", config.DefaultPool.Name))
	b.WriteString(fmt.Sprintf("    min_replicas: %d\n", config.DefaultPool.MinReplicas))
	b.WriteString(fmt.Sprintf("    max_replicas: %d\n", config.DefaultPool.MaxReplicas))
	b.WriteString(fmt.Sprintf("    target_cpu_load: %d\n", config.DefaultPool.TargetCPULoad))
	b.WriteString(fmt.Sprintf("    spot_enabled: %t\n", config.DefaultPool.SpotEnabled))
	if len(config.DefaultPool.Toolchains) > 0 {
		b.WriteString("    toolchains:\n")
		for _, tc := range config.DefaultPool.Toolchains {
			b.WriteString(fmt.Sprintf("      - %s\n", tc))
		}
	}
	if len(config.CustomPools) > 0 {
		b.WriteString("  custom_pools:\n")
		for _, pool := range config.CustomPools {
			b.WriteString(fmt.Sprintf("    - name: %s\n", pool.Name))
			b.WriteString(fmt.Sprintf("      min_replicas: %d\n", pool.MinReplicas))
			b.WriteString(fmt.Sprintf("      max_replicas: %d\n", pool.MaxReplicas))
			b.WriteString(fmt.Sprintf("      spot_enabled: %t\n", pool.SpotEnabled))
		}
	}
	return b.String()
}

func GenerateRBACManifestYAML(namespace string) string {
	var b strings.Builder
	b.WriteString("apiVersion: v1\n")
	b.WriteString("kind: ServiceAccount\n")
	b.WriteString("metadata:\n")
	b.WriteString("  name: fish-operator\n")
	b.WriteString(fmt.Sprintf("  namespace: %s\n", namespace))
	b.WriteString("---\n")
	b.WriteString("apiVersion: rbac.authorization.k8s.io/v1\n")
	b.WriteString("kind: ClusterRole\n")
	b.WriteString("metadata:\n")
	b.WriteString("  name: fish-operator-role\n")
	b.WriteString("rules:\n")
	b.WriteString("- apiGroups: [\"fish.build\"]\n")
	b.WriteString("  resources: [\"fishclusters\", \"fishclusters/status\", \"fishclusters/scale\"]\n")
	b.WriteString("  verbs: [\"get\", \"list\", \"watch\", \"create\", \"update\", \"patch\", \"delete\"]\n")
	b.WriteString("- apiGroups: [\"apps\"]\n")
	b.WriteString("  resources: [\"deployments\", \"statefulsets\"]\n")
	b.WriteString("  verbs: [\"get\", \"list\", \"watch\", \"create\", \"update\", \"patch\", \"delete\"]\n")
	b.WriteString("- apiGroups: [\"\"]\n")
	b.WriteString("  resources: [\"pods\", \"services\", \"configmaps\"]\n")
	b.WriteString("  verbs: [\"get\", \"list\", \"watch\", \"create\", \"update\", \"patch\", \"delete\"]\n")
	b.WriteString("- apiGroups: [\"autoscaling\"]\n")
	b.WriteString("  resources: [\"horizontalpodautoscalers\"]\n")
	b.WriteString("  verbs: [\"get\", \"list\", \"watch\", \"create\", \"update\"]\n")
	b.WriteString("---\n")
	b.WriteString("apiVersion: rbac.authorization.k8s.io/v1\n")
	b.WriteString("kind: ClusterRoleBinding\n")
	b.WriteString("metadata:\n")
	b.WriteString("  name: fish-operator-binding\n")
	b.WriteString("roleRef:\n")
	b.WriteString("  apiGroup: rbac.authorization.k8s.io\n")
	b.WriteString("  kind: ClusterRole\n")
	b.WriteString("  name: fish-operator-role\n")
	b.WriteString("subjects:\n")
	b.WriteString("- kind: ServiceAccount\n")
	b.WriteString("  name: fish-operator\n")
	b.WriteString(fmt.Sprintf("  namespace: %s\n", namespace))
	return b.String()
}

func GenerateOperatorDeploymentYAML(namespace string) string {
	var b strings.Builder
	b.WriteString("apiVersion: apps/v1\n")
	b.WriteString("kind: Deployment\n")
	b.WriteString("metadata:\n")
	b.WriteString("  name: fish-operator\n")
	b.WriteString(fmt.Sprintf("  namespace: %s\n", namespace))
	b.WriteString("  labels:\n")
	b.WriteString("    app: fish-operator\n")
	b.WriteString("spec:\n")
	b.WriteString("  replicas: 1\n")
	b.WriteString("  selector:\n")
	b.WriteString("    matchLabels:\n")
	b.WriteString("      app: fish-operator\n")
	b.WriteString("  template:\n")
	b.WriteString("    metadata:\n")
	b.WriteString("      labels:\n")
	b.WriteString("        app: fish-operator\n")
	b.WriteString("    spec:\n")
	b.WriteString("      serviceAccountName: fish-operator\n")
	b.WriteString("      containers:\n")
	b.WriteString("      - name: operator\n")
	b.WriteString("        image: ghcr.io/requla11/fish-operator:v0.4.0\n")
	b.WriteString("        imagePullPolicy: Always\n")
	b.WriteString("        command: [\"/manager\"]\n")
	b.WriteString("        env:\n")
	b.WriteString("        - name: WATCH_NAMESPACE\n")
	b.WriteString(fmt.Sprintf("          value: %s\n", namespace))
	b.WriteString("        - name: ENABLE_SPOT\n")
	b.WriteString("          value: \"true\"\n")
	b.WriteString("        - name: ENABLE_CROSS_REGION\n")
	b.WriteString("          value: \"true\"\n")
	b.WriteString("        resources:\n")
	b.WriteString("          limits:\n")
	b.WriteString("            cpu: 500m\n")
	b.WriteString("            memory: 512Mi\n")
	b.WriteString("          requests:\n")
	b.WriteString("            cpu: 100m\n")
	b.WriteString("            memory: 128Mi\n")
	b.WriteString("        livenessProbe:\n")
	b.WriteString("          httpGet:\n")
	b.WriteString("            path: /healthz\n")
	b.WriteString("            port: 8081\n")
	return b.String()
}
