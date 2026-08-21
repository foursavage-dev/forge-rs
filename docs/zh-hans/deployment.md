# 生产环境部署指南 (Deployment)

在云端及企业级 CI/CD 环境中部署 Fish 的完整指南。

## 部署拓扑架构
1. **单机 CI 节点**: 在 Runner 本地使用嵌入式 CAS 高速缓存。
2. **分布式集群**: 由 `fish-coordinator` 集中调度远程 Worker 节点池。
3. **Kubernetes 云原生**: 自动扩缩容 Worker Pods 并结合 Spot 实例生命周期管理。

## 默认服务端口
- **Coordinator HTTP/gRPC**: `9090`
- **Worker Gateway**: `9091`
- **P2P Swarm 网络**: `7890` (缓存) / `7891` (计算)
- **OpenTelemetry 指标**: `9094`
