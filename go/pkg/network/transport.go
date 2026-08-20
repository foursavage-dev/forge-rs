package network

import (
	"context"
	"crypto/tls"
	"encoding/binary"
	"errors"
	"fmt"
	"hash/crc32"
	"io"
	"net"
	"sync"
	"time"
)

type ConnectionPool struct {
	mu          sync.RWMutex
	connections map[string]net.Conn
	timeout     time.Duration
}

func NewConnectionPool(timeout time.Duration) *ConnectionPool {
	return &ConnectionPool{
		connections: make(map[string]net.Conn),
		timeout:     timeout,
	}
}

func (p *ConnectionPool) GetOrCreate(ctx context.Context, target string) (net.Conn, error) {
	p.mu.Lock()
	defer p.mu.Unlock()

	if conn, exists := p.connections[target]; exists {
		return conn, nil
	}

	dialer := net.Dialer{Timeout: p.timeout}
	conn, err := dialer.DialContext(ctx, "tcp", target)
	if err != nil {
		return nil, fmt.Errorf("dial failed: %w", err)
	}

	p.connections[target] = conn
	return conn, nil
}

func (p *ConnectionPool) CloseAll() {
	p.mu.Lock()
	defer p.mu.Unlock()
	for target, conn := range p.connections {
		_ = conn.Close()
		delete(p.connections, target)
	}
}

type TLSConfigBuilder struct {
	insecureSkipVerify bool
}

func NewTLSConfigBuilder() *TLSConfigBuilder {
	return &TLSConfigBuilder{insecureSkipVerify: false}
}

func (b *TLSConfigBuilder) SetInsecureSkipVerify(skip bool) *TLSConfigBuilder {
	b.insecureSkipVerify = skip
	return b
}

func (b *TLSConfigBuilder) Build() *tls.Config {
	return &tls.Config{
		MinVersion:         tls.VersionTLS13,
		InsecureSkipVerify: b.insecureSkipVerify,
	}
}

func WriteFramedPacket(w io.Writer, payload []byte) error {
	length := uint32(len(payload))
	checksum := crc32.ChecksumIEEE(payload)

	header := make([]byte, 8)
	binary.BigEndian.PutUint32(header[0:4], length)
	binary.BigEndian.PutUint32(header[4:8], checksum)

	if _, err := w.Write(header); err != nil {
		return err
	}
	if _, err := w.Write(payload); err != nil {
		return err
	}
	return nil
}

func ReadFramedPacket(r io.Reader) ([]byte, error) {
	header := make([]byte, 8)
	if _, err := io.ReadFull(r, header); err != nil {
		return nil, err
	}

	length := binary.BigEndian.Uint32(header[0:4])
	checksum := binary.BigEndian.Uint32(header[4:8])

	if length > 64*1024*1024 {
		return nil, errors.New("frame length exceeds 64MB limit")
	}

	payload := make([]byte, length)
	if _, err := io.ReadFull(r, payload); err != nil {
		return nil, err
	}

	actualChecksum := crc32.ChecksumIEEE(payload)
	if actualChecksum != checksum {
		return nil, fmt.Errorf("checksum mismatch: expected %x, got %x", checksum, actualChecksum)
	}

	return payload, nil
}
