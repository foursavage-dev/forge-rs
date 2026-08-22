# Thực Thi Từ Xa & Cache Phân Tán

Fish hỗ trợ các giao thức lưu trữ đệm phân tán và thực thi từ xa hiệu năng cao.

## Giao thức Remote Execution API (REAPI v2)
Fish tích hợp chuẩn Google / Bazel Remote Execution API v2 qua gRPC:
```bash
fish build --remote-workers 10.0.0.1:8980,10.0.0.2:8980
```

## Chia sẻ Cache P2P Swarm
Tự động khám phá và chia sẻ artifacts ngang hàng trong mạng nội bộ LAN:
```bash
fish build --swarm
```

## Lưu trữ Đám Mây CAS
Cấu hình S3, GCS, Azure Blob hoặc Redis trong `fish.toml`:
```toml
[cache]
enabled = true
remote_url = "s3://my-fish-cache-bucket"
compression = "zstd"
```
