# Hướng Dẫn Phát Hành (Release Guide)

Tài liệu này hướng dẫn quy trình phát hành phiên bản mới của hệ thống Fish.

## Các bước phát hành
1. **Kiểm định chất lượng mã nguồn**:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   ```
2. **Nâng cấp phiên bản (Version Bump)**:
   Cập nhật `Cargo.toml`, `Cargo.lock`, và `vscode-extension/package.json`.
3. **Cập nhật Nhật ký thay đổi (Changelog)**:
   Ghi nhận toàn bộ tính năng mới và bản sửa lỗi trong `CHANGELOG.md`.
4. **Tạo Git Tag**:
   ```bash
   git tag -a v0.3.0 -m "Release v0.3.0"
   git push origin v0.3.0
   ```
5. **Đóng gói & Phát hành**:
   - Đăng tải crates lên crates.io
   - Đóng gói extension VS Code `.vsix`
   - Xuất binary trên GitHub Releases
