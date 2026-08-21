package gateway

import (
	"net/http"
	"net/http/httptest"
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

	workers := gw.ListWorkers()
	if len(workers) != 1 || workers[0] != "node-1" {
		t.Fatalf("expected [node-1], got %v", workers)
	}

	req := httptest.NewRequest("GET", "/api/v1/task", nil)
	w := httptest.NewRecorder()
	gw.ServeHTTP(w, req)
	if w.Code != http.StatusBadGateway {
		t.Fatalf("expected 502 without worker header, got %d", w.Code)
	}

	gw.RemoveRoute("node-1")
	if _, exists := gw.Route("node-1"); exists {
		t.Fatal("expected route to be deleted")
	}
}
