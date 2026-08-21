# Backend Swift & Objective-C

Fish cung cấp khả năng hỗ trợ trực tiếp cho các dự án Swift và Objective-C trên macOS, iOS và Linux.

## Nhận diện tự động
Fish tự động phát hiện dự án Swift qua:
- `Package.swift` (Swift Package Manager)
- `*.xcodeproj` / `*.xcworkspace` (Dự án Xcode)

## Các lệnh hỗ trợ
```bash
fish build     # Biên dịch các module SwiftPM bằng swift build
fish test      # Chạy các bài test XCTest
fish check     # Kiểm tra cú pháp và kiểu dữ liệu với swiftc
```

## Cấu hình trong `fish.toml`
```toml
backend = "swift"
jobs = 4

[cache]
enabled = true
```
