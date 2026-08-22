package main

import (
	"fmt"
	"net/http"
)

func healthHandler(w http.ResponseWriter, r *http.Request) {
	fmt.Fprintf(w, `{"status":"ok","engine":"fish-gateway"}`)
}

func main() {
	http.HandleFunc("/health", healthHandler)
	fmt.Println("AI Gateway running on :8080")
}
