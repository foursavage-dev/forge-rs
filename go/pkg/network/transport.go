package network

import (
	"context"
	"crypto/tls"
	"fmt"
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
