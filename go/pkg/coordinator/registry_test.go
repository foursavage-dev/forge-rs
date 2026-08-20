package coordinator

import (
	"testing"
	"time"
)

func TestNodeRegistry(t *testing.T) {
	reg := NewNodeRegistry(2 * time.Second)
	node := &WorkerNode{
		ID:                  "worker-1",
		Address:             "127.0.0.1:9090",
		CPUCores:            16,
		MemoryBytes:         32 * 1024 * 1024 * 1024,
		SupportedToolchains: []string{"rust", "go"},
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

	if _, err := reg.SelectOptimalWorker("python"); err == nil {
		t.Error("expected error for unsupported toolchain")
	}
}
