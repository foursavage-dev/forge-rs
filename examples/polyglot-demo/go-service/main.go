package main

import (
	"fmt"
	"log"
	"net/http"
)

func main() {
	http.HandleFunc("/", handleRequest)

	fmt.Println("🚀 Go Service starting on port 8081...")
	log.Fatal(http.ListenAndServe(":8081", nil))
}

func handleRequest(w http.ResponseWriter, r *http.Request) {
	fmt.Fprintf(w, "Go Service: Hello from Go! Endpoint: %s", r.URL.Path)
}
