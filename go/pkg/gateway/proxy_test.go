package gateway

import (
	"testing"
)

func TestWorkerGateway(t *testing.T) {
	gw := NewWorkerGateway()
	if err := gw.AddRoute("node-1", "http://127.0.0.1:9091"); err != nil {
		t.Fatalf("failed to add route: %v", err)
	}
	proxy, exists := gw.Route("node-1")
	if !exists || proxy == nil {
		t.Fatal("expected route to exist")
	}
}
