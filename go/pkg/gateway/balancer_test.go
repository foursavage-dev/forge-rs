package gateway

import "testing"

func TestLoadBalancerLeastLoaded(t *testing.T) {
	lb := NewLoadBalancer()
	lb.AddTarget("w1", "http://worker1:9090")
	lb.AddTarget("w2", "http://worker2:9090")

	selected1, err := lb.SelectLeastLoaded()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	selected2, err := lb.SelectLeastLoaded()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if selected1.ID == selected2.ID {
		t.Fatalf("expected different workers, got %s and %s", selected1.ID, selected2.ID)
	}

	lb.Release(selected1.ID)
	selected3, err := lb.SelectLeastLoaded()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if selected3.ID != selected1.ID {
		t.Fatalf("expected worker %s to be chosen after release, got %s", selected1.ID, selected3.ID)
	}
}

func TestLoadBalancerRoundRobinAndHealth(t *testing.T) {
	lb := NewLoadBalancer()
	lb.AddTargetWithWeight("w1", "http://worker1:9090", 1)
	lb.AddTargetWithWeight("w2", "http://worker2:9090", 2)

	if lb.TargetCount() != 2 {
		t.Fatalf("expected 2 targets, got %d", lb.TargetCount())
	}

	t1, err := lb.SelectRoundRobin()
	if err != nil {
		t.Fatalf("round robin failed: %v", err)
	}
	t2, err := lb.SelectRoundRobin()
	if err != nil {
		t.Fatalf("round robin failed: %v", err)
	}
	if t1.ID == t2.ID {
		t.Fatalf("expected alternating targets in round-robin")
	}

	lb.SetHealth("w1", false)
	t3, err := lb.SelectRoundRobin()
	if err != nil || t3.ID != "w2" {
		t.Fatalf("expected only healthy target w2, got %v", t3)
	}

	lb.RemoveTarget("w2")
	_, err = lb.SelectRoundRobin()
	if err == nil {
		t.Fatal("expected error when no healthy targets remain")
	}
}
