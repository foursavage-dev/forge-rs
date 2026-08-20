package main

import (
	"net/http"
	"time"

	"github.com/requla11/fish/go/pkg/gateway"
)

func main() {
	gw := gateway.NewWorkerGateway()
	_ = gw.AddRoute("default", "http://127.0.0.1:9090")

	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"status":"ok","engine":"fish-worker-gateway"}`))
	})

	server := &http.Server{
		Addr:         ":8081",
		Handler:      mux,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 15 * time.Second,
	}
	_ = server
}
