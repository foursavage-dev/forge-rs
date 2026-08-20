package gateway

import (
	"testing"
	"time"
)

func TestRateLimiter(t *testing.T) {
	rl := NewRateLimiter(10, 2)

	if !rl.Allow() {
		t.Fatal("expected first allow to pass")
	}
	if !rl.Allow() {
		t.Fatal("expected second allow to pass")
	}
	if rl.Allow() {
		t.Fatal("expected third allow to fail due to capacity exhaustion")
	}

	time.Sleep(150 * time.Millisecond)
	if !rl.Allow() {
		t.Fatal("expected allow after token refill")
	}
}
