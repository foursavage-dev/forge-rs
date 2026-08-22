# Go 分散式服務系統 (`go/`)

Fish 包含基於 Go 1.22+ 編寫的高並發雲原生分散式服務套件。

## 核心元件
- **`fish-coordinator` (`go/cmd/fish-coordinator`)**: 節點註冊中心、優先級任務隊列及叢集心跳監控。
- **`fish-worker-gateway` (`go/cmd/fish-worker-gateway`)**: 具備權杖桶限流的負載平衡反向代理閘道器。
- **`k8s` (`go/pkg/k8s`)**: 基於 Little's Law 的 Kubernetes 自動伸縮控制器與 Spot 實例生命週期管理。
- **`mesh` (`go/pkg/mesh`)**: 具備 SHA-256 校驗與滑動視窗流量控制的 P2P Mesh 路由網路。
- **`telemetry` (`go/pkg/telemetry`)**: OpenTelemetry 分散式鏈路追蹤與 Prometheus 指標匯出器。

## 執行 Go 測試
```bash
cd go
go test -v ./...
```
