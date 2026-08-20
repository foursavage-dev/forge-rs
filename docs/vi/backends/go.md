# Go Backend

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](../TRANSLATION.md).

Go Backend cung cấp khả năng điều phối biên dịch cho các dịch vụ và công cụ viết bằng ngôn ngữ Go.

## Phát hiện Dự án (Detection)
Tự động kích hoạt khi có tệp `go.mod` trong thư mục dự án.

## Cấu hình (`fish.toml`)
```toml
[build]
backend = "go"
jobs = 8

[pipelines.build]
inputs = ["**/*.go", "go.mod", "go.sum"]
outputs = ["bin/*"]

[pipelines.test]
inputs = ["**/*.go"]
```

## Các Tác vụ Được Tạo
- **Build**: `go build -o <output> ./...`
- **Test**: `go test -v ./...`
- **Vet**: `go vet ./...`
