package network

import (
	"bytes"
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

func TestFramedPacketRoundTrip(t *testing.T) {
	payload := []byte("hello fish distributed artifact protocol")
	buf := new(bytes.Buffer)

	if err := WriteFramedPacket(buf, payload); err != nil {
		t.Fatalf("failed to write frame: %v", err)
	}

	readPayload, err := ReadFramedPacket(buf)
	if err != nil {
		t.Fatalf("failed to read frame: %v", err)
	}

	if !bytes.Equal(payload, readPayload) {
		t.Fatalf("expected payload '%s', got '%s'", payload, readPayload)
	}
}

func TestFramedPacketCorruptedChecksum(t *testing.T) {
	payload := []byte("sensitive artifact data")
	buf := new(bytes.Buffer)

	_ = WriteFramedPacket(buf, payload)

	data := buf.Bytes()
	data[len(data)-1] ^= 0xFF

	corruptedBuf := bytes.NewReader(data)
	_, err := ReadFramedPacket(corruptedBuf)
	if err == nil {
		t.Fatal("expected error on corrupted checksum")
	}
}
