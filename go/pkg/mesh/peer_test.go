package mesh

import (
	"testing"
	"time"
)

func TestP2PMeshRouter(t *testing.T) {
	router := NewP2PMeshRouter()
	router.RegisterPeer("peer-1", "10.0.0.1:9090")

	chunk := CASChunk{
		Digest:    "blake3-hash-123",
		SizeBytes: 4096,
		OwnerPeer: "peer-1",
		CreatedAt: time.Now(),
	}
	router.AnnounceChunk(chunk)

	found, err := router.LocateChunk("blake3-hash-123")
	if err != nil {
		t.Fatalf("unexpected error locating chunk: %v", err)
	}
	if found.OwnerPeer != "peer-1" {
		t.Fatalf("expected owner peer-1, got %s", found.OwnerPeer)
	}

	_, err = router.LocateChunk("missing-hash")
	if err == nil {
		t.Fatalf("expected error for missing chunk")
	}
}
