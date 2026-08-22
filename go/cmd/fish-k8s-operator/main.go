package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/requla11/fish/go/pkg/k8s"
)

func main() {
	fmt.Println("🦀 Starting Fish Kubernetes Autoscaling Operator v0.4.0...")

	config := &k8s.FishClusterConfig{
		ClusterID:       "fish-cluster-default",
		Namespace:       "fish-system",
		CoordinatorAddr: "fish-coordinator.fish-system.svc:9092",
		DefaultPool: k8s.WorkerPoolSpec{
			Name:          "fish-workers",
			MinReplicas:   1,
			MaxReplicas:   20,
			TargetCPULoad: 70,
		},
	}

	reconciler := k8s.NewClusterReconciler()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)

	go func() {
		ticker := time.NewTicker(5 * time.Second)
		defer ticker.Stop()

		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				_, _ = reconciler.Reconcile(ctx, config, 0, 1.0)
			}
		}
	}()

	fmt.Println("🚀 Fish Kubernetes Operator running. Watching custom resources (FishCluster)...")
	<-sigChan
	fmt.Println("🛑 Shutting down Fish Kubernetes Operator gracefully...")
}
