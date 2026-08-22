# Go 分布式服务系统 (`go/`)

Fish 包含基于 Go 1.22+ 编写的高并发云原生分布式服务套件。

## 核心组件
- **`fish-coordinator` (`go/cmd/fish-coordinator`)**: 节点注册中心、优先级任务队列及集群心跳监控。
- **`fish-worker-gateway` (`go/cmd/fish-worker-gateway`)**: 具备令牌桶限流的负载均衡反向代理网关。
- **`k8s` (`go/pkg/k8s`)**: 基于 Little's Law 的 Kubernetes 自动伸缩控制器与 Spot 实例生命周期管理。
- **`mesh` (`go/pkg/mesh`)**: 具备 SHA-256 校验与滑动窗口流控的 P2P Mesh 路由网络。
- **`telemetry` (`go/pkg/telemetry`)**: OpenTelemetry 分布式链路追踪与 Prometheus 指标导出器。

## 运行 Go 测试
```bash
cd go
go test -v ./...
```
