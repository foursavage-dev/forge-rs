package coordinator

import (
	"testing"
	"time"
)

func TestNodeRegistry(t *testing.T) {
	reg := NewNodeRegistry(200 * time.Millisecond)
	node := &WorkerNode{
		ID:                  "worker-1",
		Address:             "127.0.0.1:9090",
		CPUCores:            16,
		MemoryBytes:         32 * 1024 * 1024 * 1024,
		SupportedToolchains: []string{"rust", "go"},
		Tags:                map[string]string{"zone": "us-east-1", "arch": "amd64"},
	}

	if err := reg.Register(node); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	best, err := reg.SelectOptimalWorker("rust")
	if err != nil {
		t.Fatalf("failed to select worker: %v", err)
	}
	if best.ID != "worker-1" {
		t.Errorf("expected worker-1, got %s", best.ID)
	}

	tagged, err := reg.SelectWorkerWithTags("go", map[string]string{"zone": "us-east-1"})
	if err != nil {
		t.Fatalf("failed to select worker with tags: %v", err)
	}
	if tagged.ID != "worker-1" {
		t.Errorf("expected worker-1, got %s", tagged.ID)
	}

	if _, err := reg.SelectWorkerWithTags("go", map[string]string{"zone": "eu-central-1"}); err == nil {
		t.Error("expected error for mismatched tags")
	}

	if _, err := reg.SelectOptimalWorker("python"); err == nil {
		t.Error("expected error for unsupported toolchain")
	}

	if err := reg.Heartbeat("worker-1", 3); err != nil {
		t.Fatalf("heartbeat failed: %v", err)
	}

	time.Sleep(250 * time.Millisecond)
	pruned := reg.PruneExpired()
	if pruned != 1 {
		t.Errorf("expected 1 pruned worker, got %d", pruned)
	}

	if len(reg.ListHealthyWorkers()) != 0 {
		t.Errorf("expected 0 healthy workers after expiry")
	}
}

func TestNodeRegistryDeregister(t *testing.T) {
	reg := NewNodeRegistry(1 * time.Second)
	node := &WorkerNode{
		ID:                  "worker-2",
		Address:             "127.0.0.1:9091",
		SupportedToolchains: []string{"*"},
	}
	_ = reg.Register(node)
	if err := reg.Deregister("worker-2"); err != nil {
		t.Fatalf("deregister failed: %v", err)
	}
	if len(reg.ListHealthyWorkers()) != 0 {
		t.Errorf("expected 0 workers after deregister")
	}
}
