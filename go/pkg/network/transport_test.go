package network

import (
	"testing"
	"time"
)

func TestConnectionPool(t *testing.T) {
	pool := NewConnectionPool(5 * time.Second)
	if pool == nil {
		t.Fatal("expected non-nil pool")
	}
	pool.CloseAll()
}

func TestTLSConfigBuilder(t *testing.T) {
	builder := NewTLSConfigBuilder().SetInsecureSkipVerify(true)
	cfg := builder.Build()
	if cfg.MinVersion != 0x0304 {
		t.Errorf("expected TLS 1.3, got %x", cfg.MinVersion)
	}
}
