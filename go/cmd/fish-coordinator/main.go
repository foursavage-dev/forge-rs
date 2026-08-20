package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/requla11/fish/go/pkg/coordinator"
)

func main() {
	port := flag.Int("port", 8080, "Port for coordinator server")
	timeoutSec := flag.Int("timeout", 30, "Worker heartbeat timeout in seconds")
	flag.Parse()

	registry := coordinator.NewNodeRegistry(time.Duration(*timeoutSec) * time.Second)
	taskQueue := coordinator.NewTaskQueue()

	mux := http.NewServeMux()

	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		_ = json.NewEncoder(w).Encode(map[string]string{"status": "ok", "service": "fish-coordinator"})
	})

	mux.HandleFunc("/api/v1/workers", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			workers := registry.ListHealthyWorkers()
			w.Header().Set("Content-Type", "application/json")
			_ = json.NewEncoder(w).Encode(workers)
		case http.MethodPost:
			var node coordinator.WorkerNode
			if err := json.NewDecoder(r.Body).Decode(&node); err != nil {
				http.Error(w, err.Error(), http.StatusBadRequest)
				return
			}
			if err := registry.Register(&node); err != nil {
				http.Error(w, err.Error(), http.StatusBadRequest)
				return
			}
			w.WriteHeader(http.StatusCreated)
		default:
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	})

	mux.HandleFunc("/api/v1/tasks", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodPost:
			var task coordinator.QueuedTask
			if err := json.NewDecoder(r.Body).Decode(&task); err != nil {
				http.Error(w, err.Error(), http.StatusBadRequest)
				return
			}
			taskQueue.Push(task)
			w.WriteHeader(http.StatusAccepted)
		case http.MethodGet:
			toolchain := r.URL.Query().Get("toolchain")
			task, err := taskQueue.PopForToolchain(toolchain)
			if err != nil {
				http.Error(w, err.Error(), http.StatusNotFound)
				return
			}
			w.Header().Set("Content-Type", "application/json")
			_ = json.NewEncoder(w).Encode(task)
		default:
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	})

	addr := fmt.Sprintf(":%d", *port)
	server := &http.Server{
		Addr:         addr,
		Handler:      mux,
		ReadTimeout:  10 * time.Second,
		WriteTimeout: 10 * time.Second,
	}

	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)

	go func() {
		log.Printf("Fish Coordinator listening on %s", addr)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Server error: %v", err)
		}
	}()

	<-sigChan
	log.Println("Shutting down Fish Coordinator...")
}
