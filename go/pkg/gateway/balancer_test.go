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
