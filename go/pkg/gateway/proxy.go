package gateway

import (
	"fmt"
	"net/http"
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

func (g *WorkerGateway) RemoveRoute(workerID string) {
	g.mu.Lock()
	defer g.mu.Unlock()
	delete(g.routes, workerID)
	delete(g.targets, workerID)
}

func (g *WorkerGateway) Route(workerID string) (*httputil.ReverseProxy, bool) {
	g.mu.RLock()
	defer g.mu.RUnlock()
	proxy, ok := g.routes[workerID]
	return proxy, ok
}

func (g *WorkerGateway) ListWorkers() []string {
	g.mu.RLock()
	defer g.mu.RUnlock()
	workers := make([]string, 0, len(g.routes))
	for id := range g.routes {
		workers = append(workers, id)
	}
	return workers
}

func (g *WorkerGateway) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	workerID := r.Header.Get("X-Fish-Worker-ID")
	if workerID == "" {
		workerID = r.URL.Query().Get("worker_id")
	}

	proxy, ok := g.Route(workerID)
	if !ok {
		http.Error(w, "worker route not found", http.StatusBadGateway)
		return
	}
	proxy.ServeHTTP(w, r)
}
