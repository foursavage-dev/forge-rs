package k8s

import "testing"

func TestAutoscaler(t *testing.T) {
	spec := WorkerPoolSpec{
		Name:            "rust-heavy",
		MinReplicas:     2,
		MaxReplicas:     10,
		TargetCPULoad:   80,
		Toolchains:      []string{"rust", "cc"},
		ResourceLimitMB: 4096,
	}

	scaler := NewAutoscaler(spec)
	initialStatus := scaler.GetStatus()
	if initialStatus.CurrentReplicas != 2 {
		t.Fatalf("expected 2 initial replicas, got %d", initialStatus.CurrentReplicas)
	}

	desired := scaler.CalculateDesiredReplicas(50, 30.0, 10.0)
	if desired <= 2 || desired > 10 {
		t.Fatalf("unexpected desired replicas: %d", desired)
	}

	scaled, err := scaler.Scale(desired)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if scaled != desired {
		t.Fatalf("expected %d, got %d", desired, scaled)
	}

	_, err = scaler.Scale(20)
	if err == nil {
		t.Fatalf("expected out of bounds error")
	}
}
