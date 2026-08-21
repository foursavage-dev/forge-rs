package network

import "testing"

func TestBufferPool(t *testing.T) {
	pool := NewBufferPool(4096)
	buf := pool.Get()
	if buf == nil {
		t.Fatalf("expected buffer, got nil")
	}

	buf.WriteString("fish build network stream")
	if buf.String() != "fish build network stream" {
		t.Fatalf("unexpected content: %s", buf.String())
	}

	pool.Put(buf)
	buf2 := pool.Get()
	if buf2.Len() != 0 {
		t.Fatalf("expected reset buffer with len 0, got %d", buf2.Len())
	}
}
