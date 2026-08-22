# Backend Zig

Fish mang lại khả năng điều phối không overhead cho các kịch bản build Zig và chuỗi công cụ biên dịch chéo C/C++.

## Nhận diện tự động
Fish tự động phát hiện dự án Zig qua:
- `build.zig`
- `build.zig.zon`

## Các lệnh hỗ trợ
```bash
fish build     # Chạy zig build
fish test      # Chạy kiểm thử zig build test
fish check     # Xác thực cú pháp và cây AST Zig
```

## Cấu hình trong `fish.toml`
```toml
backend = "zig"
jobs = 8
```
