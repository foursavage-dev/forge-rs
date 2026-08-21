# Distributed Go Services (`go/`)

Fish contains high-concurrency cloud-native services written in Go 1.22+.

## Components
- **`fish-coordinator` (`go/cmd/fish-coordinator`)**: Central node registry, dynamic task queue with priority scheduling, and cluster heartbeat monitor.
- **`fish-worker-gateway` (`go/cmd/fish-worker-gateway`)**: Reverse proxy load balancer (Round Robin & Least Loaded) with token bucket rate limiting.
- **`k8s` (`go/pkg/k8s`)**: Kubernetes autoscaler implementing Little's Law and spot instance lifecycle manager.
- **`mesh` (`go/pkg/mesh`)**: P2P mesh network router with SHA-256 integrity verification and sliding window flow control.
- **`telemetry` (`go/pkg/telemetry`)**: OpenTelemetry distributed tracing and Prometheus metrics exporter.

## Testing Go Services
```bash
cd go
go test -v ./...
```
