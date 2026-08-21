# Rust Backend

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](../TRANSLATION.md).

Rust Backend cung cấp khả năng điều phối biên dịch cho các dự án Rust sử dụng hệ thống Cargo.

## Phát hiện Dự án (Detection)

Rust Backend được tự động kích hoạt khi có tệp `Cargo.toml` xuất hiện trong thư mục dự án.

## Cấu hình (Configuration)

Cấu hình Rust Backend thông qua `fish.toml` tại thư mục gốc của dự án hoặc workspace:

```toml
[build]
backend = "rust"
jobs = 8
no_cache = false
semantic = true
critical_path = true

[pipelines.build]
inputs = ["src/**/*", "Cargo.toml", "Cargo.lock"]
outputs = ["target/release/**/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]
```

## Các Tác vụ Được Tạo (Tasks Generated)

### Tác vụ Biên dịch (Build Task)
```bash
cargo build --release --features <features>
```

### Tác vụ Kiểm thử (Test Task)
```bash
cargo test --release --features <features>
```

### Tác vụ Kiểm tra Kiểu (Check Task)
```bash
cargo check --release --features <features>
```

### Tác vụ Tạo Tài liệu (Doc Task)
```bash
cargo doc --release --features <features>
```

## Trích xuất Phụ thuộc (Dependency Extraction)

Rust Backend trích xuất các thông tin phụ thuộc từ:
- Mục `[dependencies]` trong `Cargo.toml`
- `Cargo.lock` để lấy thông tin phiên bản chính xác
- Các phụ thuộc giữa các package trong Workspace

## Dấu vân tay Cache (Fingerprinting)

Rust Backend tính toán dấu vân tay dựa trên:
- Nội dung tệp `Cargo.toml`
- Nội dung tệp `Cargo.lock`
- Tất cả các tệp mã nguồn (loại trừ thư mục `target/`)
- Các cờ và cấu hình biên dịch

## Ví dụ Sử dụng (Examples)

### Dự án Rust Cơ bản
```bash
cd my-rust-project
fish build
```

### Workspace với các Tính năng (Features)
```bash
cd my-workspace
fish build -p my-package --features "serde,uuid"
```

### Chạy Kiểm thử trong Workspace
```bash
cd my-workspace
fish test
```

## Giới hạn & Yêu cầu
- Yêu cầu cài đặt sẵn Rust toolchain (`rustc`, `cargo`) trên hệ thống.
