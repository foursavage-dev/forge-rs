# Nhật Ký Thay Đổi & Lịch Sử Phiên Bản

Toàn bộ các thay đổi quan trọng của dự án Fish được ghi nhận tại đây.

## [v0.3.0] - 21/08/2026
### Tính năng mới
- **Plugin IDE**: Extension VS Code và Bộ Plugin JetBrains (IntelliJ / CLion / Rider).
- **Giao thức LSP**: Máy chủ `fish lsp` tự động gợi ý cú pháp và báo lỗi inline.
- **Giao thức gRPC REAPI v2**: Chuẩn Remote Execution API v2 phân tán.
- **Truy vết eBPF**: Phát hiện phụ thuộc động ở cấp nhân Linux.
- **Trợ lý Doctor AI**: Tự động chẩn đoán và khắc phục môi trường (`fish doctor --fix`).
- **Giao diện TUI Waterfall**: Giám sát CPU/RAM và tiến trình đồ thị trực quan.

## [v0.2.0] - 10/08/2026
### Tính năng mới
- **Kiến trúc Tri-Engine**: Lõi Rust 2024 + Điều phối Go + Trí tuệ nhân tạo Python.
- **11 Backend Ngôn ngữ**: Rust, Go, TS, Python, C++, Docker, Java, .NET, Swift, Dart, Zig.
- **Kho lưu trữ CAS BLAKE3**: Nén ZSTD và dọn dẹp hai pha.
- **Điều phối GNU Jobserver**: Quản lý tài nguyên CPU/RAM toàn cục.
