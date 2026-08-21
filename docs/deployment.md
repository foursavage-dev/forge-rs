# Production Deployment Guide

Comprehensive instructions for deploying Fish in cloud and production CI/CD environments.

## Deployment Topologies
1. **Single-Node Local CI**: Embedded high-speed caching on runner host.
2. **Distributed Cluster**: `fish-coordinator` managing remote worker pools.
3. **Cloud-Native Kubernetes**: Automated autoscaling worker pods with Spot instance lifecycle handling.

## Service Ports
- **Coordinator HTTP/gRPC**: `9090`
- **Worker Gateway**: `9091`
- **P2P Swarm Network**: `7890` (Cache) / `7891` (Compute)
- **OpenTelemetry Metrics**: `9094`
