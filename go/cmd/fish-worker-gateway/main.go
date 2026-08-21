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

	"github.com/requla11/fish/go/pkg/gateway"
)

func main() {
	port := flag.Int("port", 8081, "Port for worker gateway")
	flag.Parse()

	gw := gateway.NewWorkerGateway()
	lb := gateway.NewLoadBalancer()

	mux := http.NewServeMux()

	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		_ = json.NewEncoder(w).Encode(map[string]string{"status": "ok", "service": "fish-worker-gateway"})
	})

	mux.HandleFunc("/api/v1/routes", func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodPost {
			var body struct {
				ID      string `json:"id"`
				Target  string `json:"target"`
			}
			if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
				http.Error(w, err.Error(), http.StatusBadRequest)
				return
			}
			if err := gw.AddRoute(body.ID, body.Target); err != nil {
				http.Error(w, err.Error(), http.StatusBadRequest)
				return
			}
			lb.AddTarget(body.ID, body.Target)
			w.WriteHeader(http.StatusCreated)
			return
		}
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
	})

	addr := fmt.Sprintf(":%d", *port)
	server := &http.Server{
		Addr:         addr,
		Handler:      mux,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 15 * time.Second,
	}

	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)

	go func() {
		log.Printf("Fish Worker Gateway listening on %s", addr)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Gateway error: %v", err)
		}
	}()

	<-sigChan
	log.Println("Shutting down Fish Worker Gateway...")
}
