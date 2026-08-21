# Dịch Vụ Phân Tán Go (`go/`)

Fish tích hợp các dịch vụ đám mây phân tán hiệu năng cao viết bằng Go 1.22+.

## Các thành phần chính
- **`fish-coordinator` (`go/cmd/fish-coordinator`)**: Đăng ký nút mạng, hàng đợi tác vụ ưu tiên và theo dõi nhịp tim cụm (heartbeat).
- **`fish-worker-gateway` (`go/cmd/fish-worker-gateway`)**: Cổng gateway cân bằng tải (Round Robin & Least Loaded) có giới hạn tốc độ rate-limit.
- **`k8s` (`go/pkg/k8s`)**: Bộ tự động co giãn Kubernetes theo định luật Little's Law và quản lý vòng đời Spot Instance.
- **`mesh` (`go/pkg/mesh`)**: Bộ định tuyến mạng lưới P2P Mesh với kiểm tra tính toàn vẹn SHA-256.
- **`telemetry` (`go/pkg/telemetry`)**: Xuất dữ liệu truy vết OpenTelemetry và số liệu Prometheus.

## Chạy kiểm thử Go
```bash
cd go
go test -v ./...
```
