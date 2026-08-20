# Đóng góp cho Dự án Fish

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Cảm ơn bạn đã quan tâm đến việc đóng góp cho dự án Fish!

## Quy trình Đóng góp

1. Fork kho lưu trữ trên GitHub: `https://github.com/requla11/fish`
2. Clone bản fork về máy:
   ```bash
   git clone https://github.com/<YOUR_USERNAME>/fish.git
   cd fish
   git checkout -b feat/my-feature dev
   ```
3. Tiến hành chỉnh sửa theo quy chuẩn:
   - Toàn bộ định danh và thông điệp commit ghi bằng Tiếng Anh.
   - Chạy test: `cargo test --workspace`.
   - Kiểm tra định dạng: `cargo fmt --all -- --check`.
   - Kiểm tra linter: `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
4. Commit thay đổi và push lên fork của bạn.
5. Mở Pull Request hướng tới nhánh **`dev`**.
