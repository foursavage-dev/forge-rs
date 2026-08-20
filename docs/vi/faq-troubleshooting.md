# Hỏi đáp & Khắc phục sự cố

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc hoàn thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

## Các câu hỏi thường gặp

### 1. Fish khác gì so với Cargo, Turborepo, hay Bazel?
Fish được thiết kế chuyên biệt cho các dự án đa ngôn ngữ (Polyglot) với hiệu năng tiệm cận Rust gốc, hỗ trợ tính toán đồ thị thông minh bằng AI (Python) và mạng phân tán cấp đám mây (Go) mà không cần cấu hình phức tạp như Bazel.

### 2. Fish hỗ trợ những ngôn ngữ nào?
Hiện tại Fish hỗ trợ đầy đủ 11 backend ngôn ngữ: Rust, Go, TypeScript/Node.js, Python, C/C++, Docker, Java, .NET, Swift, Dart, và Zig.

### 3. Làm thế nào để kiểm tra hệ thống của tôi đã đủ công cụ chưa?
Chạy lệnh sau để kiểm tra toàn diện:
```bash
fish doctor --ai
```

## Xử lý sự cố thường gặp

### Lỗi hết bộ nhớ phân trang trên Windows (`os error 1455`)
- **Nguyên nhân:** Xảy ra khi biên dịch quá nhiều proc-macro song song làm cạn kiệt paging file của Windows.
- **Giải pháp:** Giảm số lượng jobs song song:
```bash
fish build --jobs 4
```

### Cache không trúng khi chưa sửa code
- **Nguyên nhân:** Tệp input chứa timestamp hoặc tệp tạm không mong muốn.
- **Giải pháp:** Cấu hình `inputs` glob cụ thể trong `fish.toml` để lọc bỏ các tệp rác.
