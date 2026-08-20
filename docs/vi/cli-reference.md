# Danh mục Lệnh CLI

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc hoàn thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Tài liệu tham khảo toàn bộ các lệnh và tùy chọn của dòng lệnh `fish`.

## Lệnh cơ bản

| Lệnh | Mô tả |
| :--- | :--- |
| `fish init` | Khởi tạo cấu hình `fish.toml` trong thư mục hiện tại |
| `fish new <name>` | Tạo một gói/dự án mới theo mẫu |
| `fish build` | Biên dịch toàn bộ các tác vụ trong workspace |
| `fish check` | Kiểm tra cú pháp và kiểu dữ liệu nhanh |
| `fish test` | Chạy toàn bộ các bộ kiểm thử tự động |
| `fish clean` | Xóa bỏ artifacts và giải phóng bộ nhớ đệm cục bộ |

## Lệnh AI & Trí tuệ nhân tạo

```bash
# Phân tích log lỗi build
fish ai analyze --toolchain rust --stderr "<log_content>"

# Tối ưu hóa lập lịch tác vụ DAG
fish ai optimize --workers 8

# Dự đoán các tác vụ chịu ảnh hưởng bởi thay đổi mã nguồn
fish ai recommend
```

## Lệnh Mạng & Phân tán

```bash
# Khởi chạy dịch vụ cache từ xa
fish cache-server --listen 0.0.0.0:8080

# Chạy worker node lắng nghe điều phối
fish worker --coordinator http://coordinator:9090
```

## Lệnh Phân tích & Kiểm tra

```bash
# Kiểm tra môi trường và các compiler toolchain
fish doctor --ai

# Truy vấn quan hệ phụ thuộc đồ thị
fish query "deps(//packages/core)"

# Theo dõi thay đổi tệp và tự động build
fish watch
```
