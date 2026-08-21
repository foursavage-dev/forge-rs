# 生產環境部署指南 (Deployment)

在雲端及企業級 CI/CD 環境中部署 Fish 的完整指南。

## 部署拓撲架構
1. **單機 CI 節點**: 在 Runner 本地使用嵌入式 CAS 高速快取。
2. **分散式叢集**: 由 `fish-coordinator` 集中調度遠端 Worker 節點池。
3. **Kubernetes 雲原生**: 自動擴縮容 Worker Pods 並結合 Spot 實例生命週期管理。

## 預設服務連接埠
- **Coordinator HTTP/gRPC**: `9090`
- **Worker Gateway**: `9091`
- **P2P Swarm 網路**: `7890` (快取) / `7891` (計算)
- **OpenTelemetry 指標**: `9094`
