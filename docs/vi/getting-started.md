# Bắt đầu với Fish

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc hoàn thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](TRANSLATION.md).

Hướng dẫn này sẽ giúp bạn làm quen và bắt đầu sử dụng Fish — hệ thống điều phối build siêu tốc, ưu tiên bộ nhớ đệm (cache-first).

## Cài đặt

### Cài đặt nhanh 1 dòng lệnh (Khuyến nghị)

**Linux & macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/requla11/fish/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/requla11/fish/main/install.ps1 | iex
```

### Cài đặt từ mã nguồn

```bash
git clone https://github.com/requla11/fish.git
cd fish
cargo install --path crates/fish-cli
```

### Cài đặt qua Cargo

```bash
cargo install fish-cli --git https://github.com/requla11/fish
```

## Khởi động nhanh

### Build một dự án Rust

```bash
# Di chuyển vào thư mục dự án
cd my-rust-project

# Khởi tạo tệp cấu hình fish.toml
fish init

# Thực hiện build
fish build
```

### Build dự án đa ngôn ngữ (Polyglot)

```bash
# Kiểm tra toàn bộ workspace
fish check

# Chạy toàn bộ test suites
fish test

# Dọn dẹp cache và artifacts
fish clean
```

## Khám phá giao diện TUI

Fish tích hợp sẵn giao diện dòng lệnh trực quan (Terminal UI) mạnh mẽ:

```bash
# Khởi chạy TUI tương tác
fish ui
```

## Tối ưu hóa với AI & Phân tích lỗi

```bash
# Phân tích nguyên nhân build thất bại bằng AI
fish ai analyze --toolchain rust --stderr "error[E0308]: mismatched types"

# Tối ưu hóa lịch trình đồ thị DAG
fish ai optimize --workers 8

# Đề xuất các gói cần build lại dựa trên git diff
fish ai recommend
```
