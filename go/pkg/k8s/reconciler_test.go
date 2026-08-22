package k8s

import (
	"context"
	"testing"
)

func TestClusterReconciler_Reconcile(t *testing.T) {
	config := &FishClusterConfig{
		ClusterID:       "fish-prod-cluster",
		Namespace:       "fish-system",
		CoordinatorAddr: "fish-coord.fish-system.svc:9092",
		DefaultPool: WorkerPoolSpec{
			Name:          "cpu-builders",
			MinReplicas:   2,
			MaxReplicas:   10,
			TargetCPULoad: 75,
		},
		CustomPools: []WorkerPoolSpec{
			{
				Name:          "gpu-builders",
				MinReplicas:   1,
				MaxReplicas:   4,
				TargetCPULoad: 80,
			},
		},
	}

	reconciler := NewClusterReconciler()
	res, err := reconciler.Reconcile(context.Background(), config, 25, 2.0)
	if err != nil {
		t.Fatalf("unexpected reconcile error: %v", err)
	}

	if !res.Requeue {
		t.Errorf("expected requeue to be true")
	}

	if count, ok := res.ScaledPools["cpu-builders"]; !ok || count < 2 {
		t.Errorf("expected cpu-builders to scale up, got %d", count)
	}

	if count, ok := res.ScaledPools["gpu-builders"]; !ok || count < 1 {
		t.Errorf("expected gpu-builders to scale up, got %d", count)
	}

	status, ok := reconciler.GetPoolStatus("cpu-builders")
	if !ok || status.CurrentReplicas == 0 {
		t.Errorf("expected active status for cpu-builders")
	}
}

func TestCRDManifestGeneration(t *testing.T) {
	crdYaml := GenerateCRDManifestYAML()
	if crdYaml == "" || len(crdYaml) < 50 {
		t.Errorf("expected non-empty CRD yaml manifest")
	}

	config := FishClusterConfig{
		ClusterID:       "fish-test",
		Namespace:       "default",
		CoordinatorAddr: "10.0.0.1:9092",
		DefaultPool: WorkerPoolSpec{
			Name:        "default",
			MinReplicas: 1,
			MaxReplicas: 5,
		},
	}
	deployYaml := GenerateClusterDeploymentYAML(config)
	if deployYaml == "" || len(deployYaml) < 50 {
		t.Errorf("expected non-empty cluster deployment yaml")
	}
}
