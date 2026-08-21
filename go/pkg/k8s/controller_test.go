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

func TestAutoscalerLittleLaw(t *testing.T) {
	// 10 queued tasks, 2s each, drained within 1s -> 10 tasks/s throughput
	// -> 20 workers. The old formula divided by 60 and returned 1.
	spec := WorkerPoolSpec{Name: "precise", MinReplicas: 1, MaxReplicas: 100}
	scaler := NewAutoscaler(spec)

	got := scaler.CalculateDesiredReplicas(10, 2.0, 1.0)
	if got != 20 {
		t.Fatalf("expected 20 workers, got %d", got)
	}
}
