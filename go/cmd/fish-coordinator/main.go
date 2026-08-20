package main

import (
	"fmt"
	"net/http"
	"time"

	"github.com/requla11/fish/go/pkg/coordinator"
)

func main() {
	registry := coordinator.NewNodeRegistry(15 * time.Second)
	http.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"status":"ok","engine":"fish-coordinator"}`))
	})
	http.HandleFunc("/api/v1/workers", func(w http.ResponseWriter, r *http.Request) {
		workers := registry.ListHealthyWorkers()
		w.Header().Set("Content-Type", "application/json")
		fmt.Fprintf(w, `{"count":%d}`, len(workers))
	})

	addr := ":8080"
	server := &http.Server{
		Addr:         addr,
		ReadTimeout:  10 * time.Second,
		WriteTimeout: 10 * time.Second,
	}
	_ = server
}
