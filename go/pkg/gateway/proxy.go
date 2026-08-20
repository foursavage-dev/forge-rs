package gateway

import (
	"fmt"
	"net/http/httputil"
	"net/url"
	"sync"
)

type WorkerGateway struct {
	mu      sync.RWMutex
	routes  map[string]*httputil.ReverseProxy
	targets map[string]*url.URL
}

func NewWorkerGateway() *WorkerGateway {
	return &WorkerGateway{
		routes:  make(map[string]*httputil.ReverseProxy),
		targets: make(map[string]*url.URL),
	}
}

func (g *WorkerGateway) AddRoute(workerID string, targetURL string) error {
	target, err := url.Parse(targetURL)
	if err != nil {
		return fmt.Errorf("invalid target url: %w", err)
	}

	proxy := httputil.NewSingleHostReverseProxy(target)
	g.mu.Lock()
	defer g.mu.Unlock()
	g.routes[workerID] = proxy
	g.targets[workerID] = target
	return nil
}

func (g *WorkerGateway) Route(workerID string) (*httputil.ReverseProxy, bool) {
	g.mu.RLock()
	defer g.mu.RUnlock()
	proxy, ok := g.routes[workerID]
	return proxy, ok
}
