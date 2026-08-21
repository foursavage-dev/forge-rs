# Hướng dẫn Phát triển Fish

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Tài liệu này cung cấp hướng dẫn chi tiết dành cho các nhà phát triển đóng góp vào mã nguồn của Fish.

## Điều kiện Tiên quyết

- Rust 1.88 trở lên (MSRV 1.88)
- Git
- Trình soạn thảo / IDE (khuyên dùng VS Code)
- Docker (tùy chọn, để kiểm thử container)

## Thiết lập Môi trường

```bash
# Clone kho lưu trữ
git clone https://github.com/requla11/fish.git
cd fish

# Biên dịch công cụ CLI
cargo build -p fish-cli

# Chạy toàn bộ kiểm thử
cargo test --workspace
```

## Cấu trúc Không gian làm việc

- `crates/fish-core`: Khám phá dự án, phân tích manifest, sinh compilation database.
- `crates/fish-graph`: Xây dựng đồ thị DAG, sắp xếp topo, đại số truy vấn đồ thị.
- `crates/fish-executor`: Thực thi tiến trình bất đồng bộ, tệp phản hồi, sao chép CoW nhanh.
- `crates/fish-scheduler`: Bộ lập lịch work-stealing, điều tiết tài nguyên RAM, GNU jobserver.
- `crates/fish-cache` & `fish-cas`: Tính dấu vân tay, kho lưu trữ CAS nén Zstd.
- `crates/fish-backend-*`: Các backend hỗ trợ 11+ ngôn ngữ.
- `crates/fish-cli`: Giao diện dòng lệnh CLI và bảng điều khiển web tương tác.

## Kiểm tra Chất lượng Mã nguồn

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```
