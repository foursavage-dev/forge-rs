# 分散型 Go サービス (`go/`)

Fish には Go 1.22+ で記述された高並行性クラウドネイティブサービス群が含まれます。

## 主要コンポーネント
- **`fish-coordinator` (`go/cmd/fish-coordinator`)**: ノードレジストリ、優先度付きタスクキュー、ハートビート監視。
- **`fish-worker-gateway` (`go/cmd/fish-worker-gateway`)**: トークンバケットレート制限付き負荷分散ゲートウェイ。
- **`k8s` (`go/pkg/k8s`)**: リトルの法則に基づく Kubernetes オートスケーラーおよび Spot インスタンス管理。
- **`mesh` (`go/pkg/mesh`)**: SHA-256 完全性検証付き P2P メッシュルーター。
- **`telemetry` (`go/pkg/telemetry`)**: OpenTelemetry 分散トレースおよび Prometheus エクスポーター。

## Go テストの実行
```bash
cd go
go test -v ./...
```
