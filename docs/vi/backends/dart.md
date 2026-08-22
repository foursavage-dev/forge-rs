# Backend Dart & Flutter

Fish điều phối các gói Dart CLI và ứng dụng đa nền tảng Flutter với bộ nhớ đệm tốc độ cao.

## Nhận diện tự động
Fish tự động phát hiện dự án Dart và Flutter qua:
- `pubspec.yaml`
- `pubspec.lock`

## Các lệnh hỗ trợ
```bash
fish build     # Biên dịch Dart AOT hoặc Flutter bundles
fish test      # Chạy dart test / flutter test
fish check     # Chạy phân tích tĩnh dart analyze
```

## Cấu hình trong `fish.toml`
```toml
backend = "dart"
jobs = 4
```
