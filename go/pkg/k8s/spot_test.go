package k8s

import (
	"testing"
	"time"
)

func TestSpotLifecycleManager(t *testing.T) {
	mgr := NewSpotLifecycleManager()
	mgr.RegisterTask("worker-1", "task-a")
	mgr.RegisterTask("worker-1", "task-b")

	evacuated := mgr.HandlePreemption("worker-1", 30*time.Second)
	if len(evacuated) != 2 {
		t.Fatalf("expected 2 tasks evacuated, got %d", len(evacuated))
	}
}
