# Hướng Dẫn Triển Khai Sản Xuất (Deployment)

Hướng dẫn toàn diện để triển khai Fish trên môi trường đám mây và hạ tầng CI/CD doanh nghiệp.

## Các Mô Hình Triển Khai
1. **Máy Chủ CI Độc Lập (Single-Node)**: Sử dụng bộ đệm CAS cục bộ trên máy runner.
2. **Cụm Phân Tán (Distributed Cluster)**: `fish-coordinator` điều phối nhóm máy worker từ xa.
3. **Cloud-Native Kubernetes**: Tự động co giãn pods theo tải thực tế và tối ưu chi phí Spot Instance.

## Cổng Dịch Vụ Mạng
- **Coordinator HTTP/gRPC**: `9090`
- **Worker Gateway**: `9091`
- **Mạng P2P Swarm**: `7890` (Cache) / `7891` (Compute)
- **OpenTelemetry Metrics**: `9094`
