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
}

func GenerateCRDManifestYAML() string {
	var b strings.Builder
	b.WriteString("apiVersion: apiextensions.k8s.io/v1\n")
	b.WriteString("kind: CustomResourceDefinition\n")
	b.WriteString("metadata:\n")
	b.WriteString("  name: fishclusters.fish.build\n")
	b.WriteString("spec:\n")
	b.WriteString("  group: fish.build\n")
	b.WriteString("  names:\n")
	b.WriteString("    kind: FishCluster\n")
	b.WriteString("    listKind: FishClusterList\n")
	b.WriteString("    plural: fishclusters\n")
	b.WriteString("    singular: fishcluster\n")
	b.WriteString("  scope: Namespaced\n")
	b.WriteString("  versions:\n")
	b.WriteString("    - name: v1alpha1\n")
	b.WriteString("      served: true\n")
	b.WriteString("      storage: true\n")
	b.WriteString("      schema:\n")
	b.WriteString("        openAPIV3Schema:\n")
	b.WriteString("          type: object\n")
	b.WriteString("          properties:\n")
	b.WriteString("            spec:\n")
	b.WriteString("              type: object\n")
	b.WriteString("              properties:\n")
	b.WriteString("                cluster_id:\n")
	b.WriteString("                  type: string\n")
	b.WriteString("                coordinator_addr:\n")
	b.WriteString("                  type: string\n")
	b.WriteString("                namespace:\n")
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
	b.WriteString("spec:\n")
	b.WriteString(fmt.Sprintf("  cluster_id: %s\n", config.ClusterID))
	b.WriteString(fmt.Sprintf("  coordinator_addr: %s\n", config.CoordinatorAddr))
	b.WriteString("  default_pool:\n")
	b.WriteString(fmt.Sprintf("    name: %s\n", config.DefaultPool.Name))
	b.WriteString(fmt.Sprintf("    min_replicas: %d\n", config.DefaultPool.MinReplicas))
	b.WriteString(fmt.Sprintf("    max_replicas: %d\n", config.DefaultPool.MaxReplicas))
	b.WriteString(fmt.Sprintf("    target_cpu_load: %d\n", config.DefaultPool.TargetCPULoad))
	return b.String()
}
